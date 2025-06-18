-- AI Genius Game tables for SQLite

-- Games table
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    host_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'waiting',
    max_players INTEGER NOT NULL DEFAULT 8,
    current_round INTEGER NOT NULL DEFAULT 0,
    total_rounds INTEGER NOT NULL DEFAULT 20,
    time_limit_seconds INTEGER NOT NULL DEFAULT 30,
    settings TEXT DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    started_at INTEGER,
    ended_at INTEGER,
    
    CHECK (status IN ('waiting', 'in_progress', 'completed', 'cancelled')),
    CHECK (max_players >= 2 AND max_players <= 12),
    CHECK (total_rounds >= 5 AND total_rounds <= 50)
);

CREATE INDEX IF NOT EXISTS idx_games_host_id ON games(host_id);
CREATE INDEX IF NOT EXISTS idx_games_status ON games(status);
CREATE INDEX IF NOT EXISTS idx_games_created_at ON games(created_at DESC);

-- Game players table
CREATE TABLE IF NOT EXISTS game_players (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    player_name TEXT NOT NULL,
    avatar_url TEXT,
    score INTEGER NOT NULL DEFAULT 0,
    is_ready INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    joined_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    
    UNIQUE(game_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_game_players_game_id ON game_players(game_id);
CREATE INDEX IF NOT EXISTS idx_game_players_user_id ON game_players(user_id);

-- Game rounds table
CREATE TABLE IF NOT EXISTS game_rounds (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL,
    prompt TEXT NOT NULL,
    category TEXT,
    difficulty TEXT,
    time_limit_seconds INTEGER NOT NULL,
    started_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    ended_at INTEGER,
    
    UNIQUE(game_id, round_number)
);

CREATE INDEX IF NOT EXISTS idx_game_rounds_game_id ON game_rounds(game_id);
CREATE INDEX IF NOT EXISTS idx_game_rounds_round_number ON game_rounds(round_number);

-- Player responses table
CREATE TABLE IF NOT EXISTS player_responses (
    id TEXT PRIMARY KEY,
    round_id TEXT NOT NULL REFERENCES game_rounds(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES game_players(id) ON DELETE CASCADE,
    response TEXT NOT NULL,
    response_time_ms INTEGER NOT NULL,
    is_ai_generated INTEGER NOT NULL DEFAULT 0,
    creativity_score REAL,
    humor_score REAL,
    relevance_score REAL,
    total_score REAL,
    submitted_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    
    UNIQUE(round_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_player_responses_round_id ON player_responses(round_id);
CREATE INDEX IF NOT EXISTS idx_player_responses_player_id ON player_responses(player_id);
CREATE INDEX IF NOT EXISTS idx_player_responses_total_score ON player_responses(total_score DESC);

-- Votes table
CREATE TABLE IF NOT EXISTS votes (
    id TEXT PRIMARY KEY,
    response_id TEXT NOT NULL REFERENCES player_responses(id) ON DELETE CASCADE,
    voter_id TEXT NOT NULL REFERENCES game_players(id) ON DELETE CASCADE,
    vote_type TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    
    UNIQUE(response_id, voter_id, vote_type),
    CHECK (vote_type IN ('funny', 'creative', 'best'))
);

CREATE INDEX IF NOT EXISTS idx_votes_response_id ON votes(response_id);
CREATE INDEX IF NOT EXISTS idx_votes_voter_id ON votes(voter_id);

-- Game chat messages
CREATE TABLE IF NOT EXISTS game_chat (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES game_players(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'chat',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    
    CHECK (message_type IN ('chat', 'system', 'announcement'))
);

CREATE INDEX IF NOT EXISTS idx_game_chat_game_id ON game_chat(game_id);
CREATE INDEX IF NOT EXISTS idx_game_chat_created_at ON game_chat(created_at DESC);

-- Game statistics table
CREATE TABLE IF NOT EXISTS game_stats (
    id TEXT PRIMARY KEY,
    game_id TEXT UNIQUE NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    total_responses INTEGER NOT NULL DEFAULT 0,
    ai_responses INTEGER NOT NULL DEFAULT 0,
    human_responses INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms REAL,
    avg_creativity_score REAL,
    avg_humor_score REAL,
    avg_relevance_score REAL,
    most_creative_player_id TEXT REFERENCES game_players(id),
    most_funny_player_id TEXT REFERENCES game_players(id),
    fastest_player_id TEXT REFERENCES game_players(id),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_game_stats_game_id ON game_stats(game_id);

-- Achievements table
CREATE TABLE IF NOT EXISTS achievements (
    id TEXT PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon_url TEXT,
    points INTEGER NOT NULL DEFAULT 10,
    category TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_achievements_code ON achievements(code);
CREATE INDEX IF NOT EXISTS idx_achievements_category ON achievements(category);

-- Player achievements table
CREATE TABLE IF NOT EXISTS player_achievements (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id TEXT NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    game_id TEXT REFERENCES games(id) ON DELETE SET NULL,
    earned_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    
    UNIQUE(user_id, achievement_id)
);

CREATE INDEX IF NOT EXISTS idx_player_achievements_user_id ON player_achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_player_achievements_achievement_id ON player_achievements(achievement_id);

-- Leaderboard table
CREATE TABLE IF NOT EXISTS leaderboard (
    id TEXT PRIMARY KEY,
    user_id TEXT UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total_games_played INTEGER NOT NULL DEFAULT 0,
    total_wins INTEGER NOT NULL DEFAULT 0,
    total_score INTEGER NOT NULL DEFAULT 0,
    avg_score_per_game REAL,
    highest_single_game_score INTEGER,
    total_votes_received INTEGER NOT NULL DEFAULT 0,
    creativity_votes INTEGER NOT NULL DEFAULT 0,
    humor_votes INTEGER NOT NULL DEFAULT 0,
    achievement_points INTEGER NOT NULL DEFAULT 0,
    rank INTEGER,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_leaderboard_user_id ON leaderboard(user_id);
CREATE INDEX IF NOT EXISTS idx_leaderboard_total_score ON leaderboard(total_score DESC);
CREATE INDEX IF NOT EXISTS idx_leaderboard_rank ON leaderboard(rank);

-- Trigger to update game statistics
CREATE TRIGGER IF NOT EXISTS update_game_stats_on_response
AFTER INSERT ON player_responses
BEGIN
    INSERT OR REPLACE INTO game_stats (
        id,
        game_id,
        total_responses,
        ai_responses,
        human_responses,
        avg_response_time_ms,
        avg_creativity_score,
        avg_humor_score,
        avg_relevance_score,
        updated_at
    )
    SELECT
        COALESCE(gs.id, lower(hex(randomblob(16)))),
        gr.game_id,
        COUNT(*),
        COUNT(CASE WHEN pr.is_ai_generated = 1 THEN 1 END),
        COUNT(CASE WHEN pr.is_ai_generated = 0 THEN 1 END),
        AVG(pr.response_time_ms),
        AVG(pr.creativity_score),
        AVG(pr.humor_score),
        AVG(pr.relevance_score),
        strftime('%s', 'now')
    FROM player_responses pr
    JOIN game_rounds gr ON pr.round_id = gr.id
    LEFT JOIN game_stats gs ON gs.game_id = gr.game_id
    WHERE gr.game_id = (SELECT game_id FROM game_rounds WHERE id = NEW.round_id)
    GROUP BY gr.game_id;
END;

-- Trigger to update leaderboard on game completion
CREATE TRIGGER IF NOT EXISTS update_leaderboard_on_game_end
AFTER UPDATE OF status ON games
WHEN NEW.status = 'completed'
BEGIN
    INSERT OR REPLACE INTO leaderboard (
        id,
        user_id,
        total_games_played,
        total_wins,
        total_score,
        avg_score_per_game,
        highest_single_game_score,
        updated_at
    )
    SELECT
        COALESCE(l.id, lower(hex(randomblob(16)))),
        gp.user_id,
        COUNT(DISTINCT gp.game_id),
        COUNT(DISTINCT CASE 
            WHEN gp.score = (SELECT MAX(score) FROM game_players WHERE game_id = gp.game_id) 
            THEN gp.game_id 
        END),
        SUM(gp.score),
        AVG(gp.score),
        MAX(gp.score),
        strftime('%s', 'now')
    FROM game_players gp
    LEFT JOIN leaderboard l ON l.user_id = gp.user_id
    WHERE gp.user_id IN (SELECT user_id FROM game_players WHERE game_id = NEW.id)
    GROUP BY gp.user_id;
END;

-- Insert default achievements
INSERT OR IGNORE INTO achievements (id, code, name, description, points, category) VALUES
    ('ach_001', 'first_game', 'Welcome!', 'Play your first game', 10, 'milestone'),
    ('ach_002', 'first_win', 'Winner Winner', 'Win your first game', 25, 'milestone'),
    ('ach_003', 'speed_demon', 'Speed Demon', 'Submit 5 responses in under 5 seconds each', 30, 'performance'),
    ('ach_004', 'creative_genius', 'Creative Genius', 'Receive 50 creativity votes', 50, 'social'),
    ('ach_005', 'comedian', 'Class Clown', 'Receive 50 humor votes', 50, 'social'),
    ('ach_006', 'perfect_round', 'Perfect Round', 'Get the highest score in a round', 15, 'performance'),
    ('ach_007', 'comeback_kid', 'Comeback Kid', 'Win a game after being in last place', 40, 'performance'),
    ('ach_008', 'social_butterfly', 'Social Butterfly', 'Play with 50 different players', 35, 'social'),
    ('ach_009', 'veteran', 'Veteran', 'Play 100 games', 100, 'milestone'),
    ('ach_010', 'champion', 'Champion', 'Win 50 games', 150, 'milestone');