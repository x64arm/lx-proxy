-- 添加 email 字段到 users 表
ALTER TABLE users ADD COLUMN IF NOT EXISTS email VARCHAR(255);

-- 创建 email 索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- 添加备注
COMMENT ON COLUMN users.email IS '用户邮箱，用于接收通知邮件';
