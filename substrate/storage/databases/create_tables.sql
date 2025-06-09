-- Create tables for HAL9 system
-- This is used to satisfy SQLx compile-time checking

CREATE TABLE IF NOT EXISTS signals (
    id TEXT PRIMARY KEY,
    from_neuron TEXT NOT NULL,
    to_neuron TEXT NOT NULL,
    layer_from TEXT NOT NULL,
    layer_to TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS neurons (
    id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);