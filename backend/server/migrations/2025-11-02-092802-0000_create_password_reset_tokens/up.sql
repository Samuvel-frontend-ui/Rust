CREATE TABLE password_reset_tokens (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL
);
