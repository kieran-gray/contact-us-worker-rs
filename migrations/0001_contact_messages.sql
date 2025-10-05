CREATE TABLE contact_messages (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    message TEXT NOT NULL,
    data TEXT,
    created_at BIGINT NOT NULL
);

CREATE INDEX idx_contact_messages_category ON contact_messages(category);
CREATE INDEX idx_contact_messages_created_at ON contact_messages(created_at);