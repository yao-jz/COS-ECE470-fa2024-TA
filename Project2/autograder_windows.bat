@echo off
set /p netid="Please unzip your code manually in this directory, and put it in a directory named after your netid, (make sure that Cargo.toml and src/ is in your-netid/COS-ECE470-fa2024-TA-main), and enter your netid:"
python3 add_test.py %netid%\COS-ECE470-fa2024-TA-main
cd %netid%\COS-ECE470-fa2024-TA-main
cargo test sp2022autograder01
pause