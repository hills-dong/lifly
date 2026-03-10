-- Fix admin user password hash to match "admin123".
UPDATE users SET password_hash = '$argon2id$v=19$m=19456,t=2,p=1$ZGFSdXQ8cuGgnLOhpywMwA$AQbEBqXPN1xCVOb+l1KYaeTYpEyHan0vmtKgHqd5n2c'
WHERE id = '00000000-0000-0000-0000-000000000001';
