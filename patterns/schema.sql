-- Fast Forth Pattern Library Database Schema
-- SQLite database for storing canonical Forth patterns

CREATE TABLE IF NOT EXISTS patterns (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    stack_effect TEXT NOT NULL,
    code_template TEXT NOT NULL,
    performance_class TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    usage_count INTEGER DEFAULT 0,
    success_rate REAL DEFAULT 1.0,
    CHECK (success_rate >= 0.0 AND success_rate <= 1.0)
);

CREATE TABLE IF NOT EXISTS pattern_tags (
    pattern_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (pattern_id, tag),
    FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS pattern_test_cases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_id TEXT NOT NULL,
    input_values TEXT NOT NULL,  -- JSON array
    output_values TEXT NOT NULL, -- JSON array
    description TEXT,
    FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS template_variables (
    pattern_id TEXT NOT NULL,
    variable_name TEXT NOT NULL,
    description TEXT,
    example TEXT,
    required BOOLEAN DEFAULT 1,
    PRIMARY KEY (pattern_id, variable_name),
    FOREIGN KEY (pattern_id) REFERENCES patterns(id) ON DELETE CASCADE
);

-- Indexes for fast queries
CREATE INDEX IF NOT EXISTS idx_patterns_category ON patterns(category);
CREATE INDEX IF NOT EXISTS idx_patterns_stack_effect ON patterns(stack_effect);
CREATE INDEX IF NOT EXISTS idx_patterns_performance ON patterns(performance_class);
CREATE INDEX IF NOT EXISTS idx_pattern_tags_tag ON pattern_tags(tag);
CREATE INDEX IF NOT EXISTS idx_pattern_tags_pattern ON pattern_tags(pattern_id);

-- Metadata table
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

INSERT INTO schema_version (version, applied_at) VALUES (1, datetime('now'));
