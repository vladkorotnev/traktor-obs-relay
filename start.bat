@echo off
start D:\Chrome\chrome.exe --disable-background-timer-throttling "http://127.0.0.1:8080/vfd.html" "http://127.0.0.1:8080/logger.html"
cargo run