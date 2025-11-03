CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    phoneno VARCHAR(20) NOT NULL,
    account_type VARCHAR(10) NOT NULL,
    profile_pic VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);