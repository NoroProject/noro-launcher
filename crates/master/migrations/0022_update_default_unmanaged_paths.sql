-- Xaero waypoints and the OptiFine settings files are player data; sync has to
-- leave them alone.
ALTER TABLE builds
ALTER COLUMN unmanaged_paths SET DEFAULT '["saves/","screenshots/","options.txt","optionsof.txt","optionsshaders.txt","logs/","crash-reports/","xaero*","config/xaero*","xaerominimap*","xaeroworldmap*"]'::jsonb;

-- Backfill existing builds. The guard only tests for xaero*, so a build that
-- already lists it is left as-is even if it misses the other patterns.
UPDATE builds
SET unmanaged_paths = unmanaged_paths || '["xaero*","config/xaero*","xaerominimap*","xaeroworldmap*","optionsof.txt","optionsshaders.txt"]'::jsonb
WHERE NOT (unmanaged_paths @> '["xaero*"]'::jsonb);
