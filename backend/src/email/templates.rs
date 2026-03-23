/// 邮件模板模块

/// 邮件模板类型
pub enum EmailTemplate {
    TrafficAlert,
    ExpiryReminder,
    DisabledNotification,
    Test,
}

/// 流量告警邮件模板
pub fn traffic_alert(inbound_name: &str, usage_percent: f64) -> String {
    let color = if usage_percent >= 90.0 {
        "#dc3545"
    } else if usage_percent >= 70.0 {
        "#ffc107"
    } else {
        "#28a745"
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: {}; color: white; padding: 20px; text-align: center; border-radius: 5px 5px 0 0; }}
        .content {{ background: #f9f9f9; padding: 30px; border: 1px solid #ddd; }}
        .alert-box {{ background: {}; color: white; padding: 15px; border-radius: 5px; margin: 20px 0; text-align: center; }}
        .progress {{ background: #e0e0e0; border-radius: 10px; overflow: hidden; margin: 20px 0; }}
        .progress-bar {{ background: {}; height: 30px; width: {}%; text-align: center; line-height: 30px; color: white; font-weight: bold; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header" style="background: {};">
            <h1>⚠️ 流量告警</h1>
        </div>
        <div class="content">
            <p>您好，</p>
            <p>您的代理配置 <strong>{}</strong> 流量使用已达到警戒值。</p>
            
            <div class="alert-box" style="background: {};">
                <h2 style="margin: 0;">已使用 {:.1}%</h2>
            </div>
            
            <div class="progress">
                <div class="progress-bar" style="background: {}; width: {}%;">{:.1}%</div>
            </div>
            
            <p>请及时关注流量使用情况，避免服务中断。</p>
            <p>如需增加流量限额，请联系管理员。</p>
        </div>
        <div class="footer">
            <p>此邮件由 LX-Proxy 系统自动发送</p>
            <p>请勿直接回复此邮件</p>
        </div>
    </div>
</body>
</html>"#,
        color, color, color, usage_percent,
        color, inbound_name, color, usage_percent,
        color, usage_percent, usage_percent
    )
}

/// 到期提醒邮件模板
pub fn expiry_reminder(inbound_name: &str, days_left: u32) -> String {
    let urgency = if days_left <= 3 {
        "紧急"
    } else if days_left <= 7 {
        "重要"
    } else {
        "提醒"
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #17a2b8; color: white; padding: 20px; text-align: center; border-radius: 5px 5px 0 0; }}
        .content {{ background: #f9f9f9; padding: 30px; border: 1px solid #ddd; }}
        .info-box {{ background: #e7f3ff; border-left: 4px solid #17a2b8; padding: 15px; margin: 20px 0; }}
        .countdown {{ font-size: 36px; font-weight: bold; color: #17a2b8; text-align: center; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⏰ {}提醒</h1>
        </div>
        <div class="content">
            <p>您好，</p>
            <p>您的代理配置 <strong>{}</strong> 即将到期。</p>
            
            <div class="info-box">
                <p style="margin: 0;">配置名称：{}</p>
                <p style="margin: 10px 0 0 0;">剩余时间：</p>
            </div>
            
            <div class="countdown">
                {} 天
            </div>
            
            <p>请及时续费或联系管理员延长服务时间，避免服务中断。</p>
        </div>
        <div class="footer">
            <p>此邮件由 LX-Proxy 系统自动发送</p>
            <p>请勿直接回复此邮件</p>
        </div>
    </div>
</body>
</html>"#,
        urgency, inbound_name, inbound_name, days_left
    )
}

/// 禁用通知邮件模板
pub fn disabled_notification(inbound_name: &str, reason: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #dc3545; color: white; padding: 20px; text-align: center; border-radius: 5px 5px 0 0; }}
        .content {{ background: #f9f9f9; padding: 30px; border: 1px solid #ddd; }}
        .alert-box {{ background: #f8d7da; border: 1px solid #f5c6cb; color: #721c24; padding: 15px; border-radius: 5px; margin: 20px 0; }}
        .reason {{ background: white; padding: 15px; border-left: 4px solid #dc3545; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚫 服务已禁用</h1>
        </div>
        <div class="content">
            <p>您好，</p>
            <p>您的代理配置 <strong>{}</strong> 已被系统自动禁用。</p>
            
            <div class="alert-box">
                <strong>⚠️ 服务已停止</strong>
            </div>
            
            <div class="reason">
                <p style="margin: 0;"><strong>禁用原因：</strong></p>
                <p style="margin: 10px 0 0 0;">{}</p>
            </div>
            
            <p>如需恢复服务，请联系管理员处理。</p>
        </div>
        <div class="footer">
            <p>此邮件由 LX-Proxy 系统自动发送</p>
            <p>请勿直接回复此邮件</p>
        </div>
    </div>
</body>
</html>"#,
        inbound_name, reason
    )
}

/// 测试邮件模板
pub fn test_email() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
        .container { max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #28a745; color: white; padding: 20px; text-align: center; border-radius: 5px 5px 0 0; }
        .content { background: #f9f9f9; padding: 30px; border: 1px solid #ddd; }
        .success-box { background: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .footer { text-align: center; padding: 20px; color: #666; font-size: 12px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>✅ 邮件测试成功</h1>
        </div>
        <div class="content">
            <div class="success-box">
                <strong>🎉 恭喜！</strong>
                <p style="margin: 10px 0 0 0;">LX-Proxy 邮件系统配置正确，可以正常发送邮件。</p>
            </div>
            <p>如果您收到此邮件，说明 SMTP 配置已成功验证。</p>
        </div>
        <div class="footer">
            <p>此邮件由 LX-Proxy 系统自动发送</p>
            <p>请勿直接回复此邮件</p>
        </div>
    </div>
</body>
</html>"#.to_string()
}
