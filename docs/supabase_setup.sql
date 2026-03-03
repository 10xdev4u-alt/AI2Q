-- Supabase Setup for AIQL
-- This RPC allows the Edge Function to execute generated SQL safely.

CREATE OR REPLACE FUNCTION execute_ai_query(query_text TEXT)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER -- Runs with privileges of the creator
AS $$
DECLARE
    result JSONB;
BEGIN
    -- WARNING: This is a powerful function. 
    -- In production, you should use AIQL's Read-Only enforcement 
    -- and potentially a dedicated restricted user.
    EXECUTE 'SELECT jsonb_agg(t) FROM (' || query_text || ') t' INTO result;
    RETURN result;
END;
$$;
