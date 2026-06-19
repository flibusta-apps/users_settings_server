-- Add file name language setting: 'normalized' (default, Latin transliteration) or 'original' (use source language)
ALTER TABLE user_settings
ADD COLUMN file_name_lang VARCHAR(16) NOT NULL DEFAULT 'normalized'
CHECK (file_name_lang IN ('normalized', 'original'));
