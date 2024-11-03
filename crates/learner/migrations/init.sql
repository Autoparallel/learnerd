-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Create papers table
CREATE TABLE IF NOT EXISTS papers (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    abstract_text TEXT NOT NULL,
    publication_date TEXT NOT NULL,  -- Stored as ISO8601
    source TEXT NOT NULL,
    source_identifier TEXT NOT NULL,
    pdf_url TEXT,
    doi TEXT,
    metadata TEXT,  -- JSON storage
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source, source_identifier)
);

-- Create authors table
CREATE TABLE IF NOT EXISTS authors (
    id INTEGER PRIMARY KEY,
    paper_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    affiliation TEXT,
    email TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY(paper_id) REFERENCES papers(id) ON DELETE CASCADE
);

-- Full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS papers_fts USING fts5(
    title, 
    abstract_text,
    content=papers,
    content_rowid=id
);

-- Triggers to keep FTS index updated
CREATE TRIGGER IF NOT EXISTS papers_ai AFTER INSERT ON papers BEGIN
    INSERT INTO papers_fts(rowid, title, abstract_text)
    VALUES (new.id, new.title, new.abstract_text);
END;

CREATE TRIGGER IF NOT EXISTS papers_ad AFTER DELETE ON papers BEGIN
    INSERT INTO papers_fts(papers_fts, rowid, title, abstract_text)
    VALUES('delete', old.id, old.title, old.abstract_text);
END;

CREATE TRIGGER IF NOT EXISTS papers_au AFTER UPDATE ON papers BEGIN
    INSERT INTO papers_fts(papers_fts, rowid, title, abstract_text)
    VALUES('delete', old.id, old.title, old.abstract_text);
    INSERT INTO papers_fts(rowid, title, abstract_text)
    VALUES (new.id, new.title, new.abstract_text);
END;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_papers_source_id ON papers(source, source_identifier);
CREATE INDEX IF NOT EXISTS idx_papers_doi ON papers(doi) WHERE doi IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_authors_paper_id ON authors(paper_id);
CREATE INDEX IF NOT EXISTS idx_authors_name ON authors(name);