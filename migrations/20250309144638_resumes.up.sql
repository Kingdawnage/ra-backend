-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "resumes" (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    user_id UUID NOT NULL REFERENCES "users" (id) ON DELETE CASCADE,
    file_path VARCHAR(255) NOT NULL,
    analysis_result JSONB,
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);