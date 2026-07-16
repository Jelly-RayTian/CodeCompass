-- V9: Adds line_count to indexed_files for complexity metrics.
--
-- The analysis runner counts source lines during parser execution and
-- stores the value here. Pre-existing files (scanned before V9) will
-- show line_count = 0 until re-analyzed.

ALTER TABLE indexed_files ADD COLUMN line_count INTEGER NOT NULL DEFAULT 0;
