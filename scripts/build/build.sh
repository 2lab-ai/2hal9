#!/bin/bash
# Build script with SQLx offline mode enabled

export SQLX_OFFLINE=true
export DATABASE_URL=sqlite:///Users/icedac/2lab.ai/2hal9/substrate/storage/databases/hal9.db?mode=rwc

cargo build "$@"