use crate::error::{LumenError, LumenResult as Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::state::TranscriptionRecord;

/// Responsável pela persistência e análise do histórico de transcrições.
/// Usa SQLite (rusqlite local) armazenado em ~/.local/share/lumen/analytics.db
pub struct Analytics {
    db: Mutex<Connection>,
}

impl Analytics {
    /// Inicializa a conexão com o banco de dados e cria as tabelas caso não existam.
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let db_dir = db_path.parent().ok_or_else(|| LumenError::Internal("Caminho do DB inválido".into()))?;

        if !db_dir.exists() {
            std::fs::create_dir_all(db_dir)
                .map_err(|e| LumenError::Internal(format!("Falha ao criar diretório: {}", e)))?;
        }

        let db = Connection::open(&db_path)
            .map_err(|e| LumenError::Internal(format!("Falha ao abrir conexão SQLite: {}", e)))?;

        // Inicializar schema
        db.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                raw_text TEXT NOT NULL,
                processed_text TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                processing_time_ms INTEGER NOT NULL,
                ai_used BOOLEAN NOT NULL,
                auto_sent BOOLEAN NOT NULL
            )",
            [],
        ).map_err(|e| LumenError::Internal(format!("Falha ao criar tabela: {}", e)))?;

        // Índices para consultas futuras (paginação, busca por data)
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcriptions_timestamp 
             ON transcriptions (timestamp DESC)",
            [],
        ).map_err(|e| LumenError::Internal(format!("Falha index: {}", e)))?;

        tracing::info!("📊 Analytics DB sincronizado em: {}", db_path.display());

        Ok(Self { db: Mutex::new(db) })
    }

    /// Executa inicialização padronizada, criando DB no path de dados do Lumen.
    pub fn init_default() -> Result<Self> {
        let mut path = crate::config::LumenConfig::data_dir();
        path.push("analytics.db");
        Self::new(path)
    }

    /// Salva um registro completo de transcrição no banco de dados.
    pub fn save_transcription(&self, record: &TranscriptionRecord) -> Result<()> {
        let timestamp_str = record.timestamp.to_rfc3339();

        let db = self.db.lock().map_err(|_| LumenError::Internal("Mutex envenenado".into()))?;
        db.execute(
            "INSERT INTO transcriptions (
                id, timestamp, raw_text, processed_text, 
                word_count, processing_time_ms, ai_used, auto_sent
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                timestamp_str,
                record.raw_text,
                record.processed_text,
                record.word_count,
                record.processing_time_ms,
                record.ai_used,
                record.auto_sent
            ],
        ).map_err(|e| LumenError::Internal(format!("Falha db: {}", e)))?;

        Ok(())
    }

    /// Remove uma transcrição do histórico pelo ID.
    pub fn delete_transcription(&self, id: &str) -> Result<bool> {
        let db = self.db.lock().map_err(|_| LumenError::Internal("Mutex envenenado".into()))?;
        let rows = db.execute(
            "DELETE FROM transcriptions WHERE id = ?1",
            params![id],
        )?;
        Ok(rows > 0)
    }

    /// Limpa todo o banco de histórico (usado para o botão "Limpar Histórico").
    pub fn clear_history(&self) -> Result<()> {
        let db = self.db.lock().map_err(|_| LumenError::Internal("Mutex envenenado".into()))?;
        db.execute("DELETE FROM transcriptions", [])?;
        Ok(())
    }

    /// Retorna as transcrições mais recentes com paginação.
    pub fn get_recent_transcriptions(&self, limit: usize, offset: usize) -> Result<Vec<TranscriptionRecord>> {
        let db = self.db.lock().map_err(|_| LumenError::Internal("Mutex envenenado".into()))?;
        let mut stmt = db.prepare(
            "SELECT id, timestamp, raw_text, processed_text, 
                    word_count, processing_time_ms, ai_used, auto_sent 
             FROM transcriptions 
             ORDER BY timestamp DESC 
             LIMIT ?1 OFFSET ?2"
        ).map_err(|e| LumenError::Internal(format!("DB stmt err: {}", e)))?;

        let db_iter = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_else(|_| chrono::Local::now().into())
                .with_timezone(&Utc);

            Ok(TranscriptionRecord {
                id: row.get(0)?,
                timestamp,
                raw_text: row.get(2)?,
                processed_text: row.get(3)?,
                word_count: row.get(4)?,
                processing_time_ms: row.get(5)?,
                ai_used: row.get(6)?,
                auto_sent: row.get(7)?,
            })
        })?;

        let mut records = Vec::new();
        for record in db_iter {
            if let Ok(r) = record {
                records.push(r);
            }
        }

        Ok(records)
    }

    /// Obtém estatísticas globais precomputadas (lifetime).
    pub fn get_global_stats(&self) -> Result<(u64, u64)> {
        let db = self.db.lock().map_err(|_| LumenError::Internal("Mutex envenenado".into()))?;
        let mut stmt = db.prepare(
            "SELECT COUNT(id), COALESCE(SUM(word_count), 0) FROM transcriptions"
        )?;

        let mut rows = stmt.query([]).map_err(|e| LumenError::Internal(format!("DB query err: {}", e)))?;
        
        if let Some(row) = rows.next()? {
            let total_docs: i64 = row.get(0)?;
            let total_words: i64 = row.get(1)?;
            Ok((total_docs as u64, total_words as u64))
        } else {
            Ok((0, 0))
        }
    }
}
