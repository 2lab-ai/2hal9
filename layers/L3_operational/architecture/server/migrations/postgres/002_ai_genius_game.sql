-- AI Genius Game tables for PostgreSQL

-- Games table
CREATE TABLE IF NOT EXISTS games (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    host_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'waiting',
    max_players INTEGER NOT NULL DEFAULT 8,
    current_round INTEGER NOT NULL DEFAULT 0,
    total_rounds INTEGER NOT NULL DEFAULT 20,
    time_limit_seconds INTEGER NOT NULL DEFAULT 30,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    
    CONSTRAINT valid_status CHECK (status IN ('waiting', 'in_progress', 'completed', 'cancelled')),
    CONSTRAINT valid_players CHECK (max_players >= 2 AND max_players <= 12),
    CONSTRAINT valid_rounds CHECK (total_rounds >= 5 AND total_rounds <= 50)
);

CREATE INDEX idx_games_host_id ON games(host_id);
CREATE INDEX idx_games_status ON games(status);
CREATE INDEX idx_games_created_at ON games(created_at DESC);

-- Game players table
CREATE TABLE IF NOT EXISTS game_players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    player_name VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    score INTEGER NOT NULL DEFAULT 0,
    is_ready BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(game_id, user_id)
);

CREATE INDEX idx_game_players_game_id ON game_players(game_id);
CREATE INDEX idx_game_players_user_id ON game_players(user_id);

-- Game rounds table
CREATE TABLE IF NOT EXISTS game_rounds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL,
    prompt TEXT NOT NULL,
    category VARCHAR(100),
    difficulty VARCHAR(50),
    time_limit_seconds INTEGER NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    
    UNIQUE(game_id, round_number)
);

CREATE INDEX idx_game_rounds_game_id ON game_rounds(game_id);
CREATE INDEX idx_game_rounds_round_number ON game_rounds(round_number);

-- Player responses table
CREATE TABLE IF NOT EXISTS player_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    round_id UUID NOT NULL REFERENCES game_rounds(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES game_players(id) ON DELETE CASCADE,
    response TEXT NOT NULL,
    response_time_ms INTEGER NOT NULL,
    is_ai_generated BOOLEAN NOT NULL DEFAULT false,
    creativity_score FLOAT,
    humor_score FLOAT,
    relevance_score FLOAT,
    total_score FLOAT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(round_id, player_id)
);

CREATE INDEX idx_player_responses_round_id ON player_responses(round_id);
CREATE INDEX idx_player_responses_player_id ON player_responses(player_id);
CREATE INDEX idx_player_responses_total_score ON player_responses(total_score DESC);

-- Votes table
CREATE TABLE IF NOT EXISTS votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    response_id UUID NOT NULL REFERENCES player_responses(id) ON DELETE CASCADE,
    voter_id UUID NOT NULL REFERENCES game_players(id) ON DELETE CASCADE,
    vote_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(response_id, voter_id, vote_type),
    CONSTRAINT valid_vote_type CHECK (vote_type IN ('funny', 'creative', 'best'))
);

CREATE INDEX idx_votes_response_id ON votes(response_id);
CREATE INDEX idx_votes_voter_id ON votes(voter_id);

-- Game chat messages
CREATE TABLE IF NOT EXISTS game_chat (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES game_players(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    message_type VARCHAR(50) NOT NULL DEFAULT 'chat',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_message_type CHECK (message_type IN ('chat', 'system', 'announcement'))
);

CREATE INDEX idx_game_chat_game_id ON game_chat(game_id);
CREATE INDEX idx_game_chat_created_at ON game_chat(created_at DESC);

-- Game statistics table
CREATE TABLE IF NOT EXISTS game_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id UUID UNIQUE NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    total_responses INTEGER NOT NULL DEFAULT 0,
    ai_responses INTEGER NOT NULL DEFAULT 0,
    human_responses INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms FLOAT,
    avg_creativity_score FLOAT,
    avg_humor_score FLOAT,
    avg_relevance_score FLOAT,
    most_creative_player_id UUID REFERENCES game_players(id),
    most_funny_player_id UUID REFERENCES game_players(id),
    fastest_player_id UUID REFERENCES game_players(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_game_stats_game_id ON game_stats(game_id);

-- Achievements table
CREATE TABLE IF NOT EXISTS achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    icon_url TEXT,
    points INTEGER NOT NULL DEFAULT 10,
    category VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_achievements_code ON achievements(code);
CREATE INDEX idx_achievements_category ON achievements(category);

-- Player achievements table
CREATE TABLE IF NOT EXISTS player_achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id UUID NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    game_id UUID REFERENCES games(id) ON DELETE SET NULL,
    earned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, achievement_id)
);

CREATE INDEX idx_player_achievements_user_id ON player_achievements(user_id);
CREATE INDEX idx_player_achievements_achievement_id ON player_achievements(achievement_id);

-- Leaderboard table (materialized view would be better for production)
CREATE TABLE IF NOT EXISTS leaderboard (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total_games_played INTEGER NOT NULL DEFAULT 0,
    total_wins INTEGER NOT NULL DEFAULT 0,
    total_score INTEGER NOT NULL DEFAULT 0,
    avg_score_per_game FLOAT,
    highest_single_game_score INTEGER,
    total_votes_received INTEGER NOT NULL DEFAULT 0,
    creativity_votes INTEGER NOT NULL DEFAULT 0,
    humor_votes INTEGER NOT NULL DEFAULT 0,
    achievement_points INTEGER NOT NULL DEFAULT 0,
    rank INTEGER,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_leaderboard_user_id ON leaderboard(user_id);
CREATE INDEX idx_leaderboard_total_score ON leaderboard(total_score DESC);
CREATE INDEX idx_leaderboard_rank ON leaderboard(rank);

-- Function to update game statistics
CREATE OR REPLACE FUNCTION update_game_stats(p_game_id UUID)
RETURNS void AS $$
BEGIN
    INSERT INTO game_stats (
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
        p_game_id,
        COUNT(*),
        COUNT(*) FILTER (WHERE pr.is_ai_generated = true),
        COUNT(*) FILTER (WHERE pr.is_ai_generated = false),
        AVG(pr.response_time_ms),
        AVG(pr.creativity_score),
        AVG(pr.humor_score),
        AVG(pr.relevance_score),
        NOW()
    FROM player_responses pr
    JOIN game_rounds gr ON pr.round_id = gr.id
    WHERE gr.game_id = p_game_id
    ON CONFLICT (game_id) DO UPDATE SET
        total_responses = EXCLUDED.total_responses,
        ai_responses = EXCLUDED.ai_responses,
        human_responses = EXCLUDED.human_responses,
        avg_response_time_ms = EXCLUDED.avg_response_time_ms,
        avg_creativity_score = EXCLUDED.avg_creativity_score,
        avg_humor_score = EXCLUDED.avg_humor_score,
        avg_relevance_score = EXCLUDED.avg_relevance_score,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Function to update leaderboard
CREATE OR REPLACE FUNCTION update_leaderboard(p_user_id UUID)
RETURNS void AS $$
BEGIN
    INSERT INTO leaderboard (
        user_id,
        total_games_played,
        total_wins,
        total_score,
        avg_score_per_game,
        highest_single_game_score,
        total_votes_received,
        creativity_votes,
        humor_votes,
        achievement_points,
        updated_at
    )
    SELECT
        p_user_id,
        COUNT(DISTINCT gp.game_id),
        COUNT(DISTINCT gp.game_id) FILTER (WHERE gp.score = (
            SELECT MAX(score) FROM game_players WHERE game_id = gp.game_id
        )),
        COALESCE(SUM(gp.score), 0),
        COALESCE(AVG(gp.score), 0),
        COALESCE(MAX(gp.score), 0),
        COUNT(DISTINCT v.id),
        COUNT(DISTINCT v.id) FILTER (WHERE v.vote_type = 'creative'),
        COUNT(DISTINCT v.id) FILTER (WHERE v.vote_type = 'funny'),
        COALESCE(SUM(DISTINCT a.points), 0),
        NOW()
    FROM game_players gp
    LEFT JOIN player_responses pr ON pr.player_id = gp.id
    LEFT JOIN votes v ON v.response_id = pr.id
    LEFT JOIN player_achievements pa ON pa.user_id = p_user_id
    LEFT JOIN achievements a ON a.id = pa.achievement_id
    WHERE gp.user_id = p_user_id
    GROUP BY gp.user_id
    ON CONFLICT (user_id) DO UPDATE SET
        total_games_played = EXCLUDED.total_games_played,
        total_wins = EXCLUDED.total_wins,
        total_score = EXCLUDED.total_score,
        avg_score_per_game = EXCLUDED.avg_score_per_game,
        highest_single_game_score = EXCLUDED.highest_single_game_score,
        total_votes_received = EXCLUDED.total_votes_received,
        creativity_votes = EXCLUDED.creativity_votes,
        humor_votes = EXCLUDED.humor_votes,
        achievement_points = EXCLUDED.achievement_points,
        updated_at = NOW();
    
    -- Update ranks
    WITH ranked_users AS (
        SELECT 
            user_id,
            ROW_NUMBER() OVER (ORDER BY total_score DESC, total_wins DESC) as new_rank
        FROM leaderboard
    )
    UPDATE leaderboard l
    SET rank = ru.new_rank
    FROM ranked_users ru
    WHERE l.user_id = ru.user_id;
END;
$$ LANGUAGE plpgsql;

-- Insert default achievements
INSERT INTO achievements (code, name, description, points, category) VALUES
    ('first_game', 'Welcome!', 'Play your first game', 10, 'milestone'),
    ('first_win', 'Winner Winner', 'Win your first game', 25, 'milestone'),
    ('speed_demon', 'Speed Demon', 'Submit 5 responses in under 5 seconds each', 30, 'performance'),
    ('creative_genius', 'Creative Genius', 'Receive 50 creativity votes', 50, 'social'),
    ('comedian', 'Class Clown', 'Receive 50 humor votes', 50, 'social'),
    ('perfect_round', 'Perfect Round', 'Get the highest score in a round', 15, 'performance'),
    ('comeback_kid', 'Comeback Kid', 'Win a game after being in last place', 40, 'performance'),
    ('social_butterfly', 'Social Butterfly', 'Play with 50 different players', 35, 'social'),
    ('veteran', 'Veteran', 'Play 100 games', 100, 'milestone'),
    ('champion', 'Champion', 'Win 50 games', 150, 'milestone')
ON CONFLICT (code) DO NOTHING;