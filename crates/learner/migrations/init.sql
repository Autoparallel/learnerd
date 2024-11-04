-- Add a configuration table for global settings
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
) STRICT;

-- Add a files table to track paper files
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY,
    paper_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    filename TEXT NOT NULL,
    hash TEXT,  -- For future integrity checking
    last_modified DATETIME NOT NULL,
    file_type TEXT NOT NULL,  -- e.g., 'pdf', future-proofing for other types
    FOREIGN KEY(paper_id) REFERENCES papers(id) ON DELETE CASCADE,
    UNIQUE(paper_id, file_type)  -- One file type per paper
) STRICT;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_files_paper_id ON files(paper_id);

-- Add full-text search support for filenames
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    filename,
    content='files',
    content_rowid='id'
);

-- Add triggers to maintain FTS index
CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
    INSERT INTO files_fts(rowid, filename) VALUES (new.id, new.filename);
END;

CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, filename) VALUES('delete', old.id, old.filename);
END;

CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, filename) VALUES('delete', old.id, old.filename);
    INSERT INTO files_fts(rowid, filename) VALUES (new.id, new.filename);
END;