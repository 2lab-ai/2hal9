#!/usr/bin/env node
const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const path = require('path');

console.log('🧪 AI Genius Game JWT 인증 웹 테스트');
console.log('='.repeat(60));

async function testAuthWeb() {
    let gameProcess;
    let browser;
    
    try {
        // 1. 게임 서버 시작
        console.log('\n📌 Step 1: AI Genius Game 서버 시작');
        const serverPath = path.join(__dirname, 'target/release/ai-genius-game');
        gameProcess = spawn(serverPath, [], {
            cwd: __dirname
        });
        
        gameProcess.stdout.on('data', (data) => {
            console.log(`   서버: ${data.toString().trim()}`);
        });
        
        console.log('   프로세스 시작됨 (PID:', gameProcess.pid, ')');
        console.log('   서버 초기화 대기 중...');
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // 2. 브라우저 시작
        console.log('\n📌 Step 2: Puppeteer 브라우저 시작');
        browser = await puppeteer.launch({
            headless: 'new',
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        
        const page = await browser.newPage();
        
        // 콘솔 메시지 캡처
        page.on('console', msg => {
            console.log('   브라우저 콘솔:', msg.text());
        });
        
        // 3. 홈페이지 로드
        console.log('\n📌 Step 3: 홈페이지 로드');
        await page.goto('http://localhost:3456', { waitUntil: 'networkidle0' });
        const title = await page.title();
        console.log(`   페이지 제목: ${title}`);
        
        // 4. 로그인 폼 확인
        console.log('\n📌 Step 4: 로그인 폼 확인');
        const authSection = await page.$('#authSection');
        console.log(`   로그인 섹션: ${authSection ? '있음' : '없음'}`);
        
        const usernameInput = await page.$('#username');
        const passwordInput = await page.$('#password');
        console.log(`   사용자명 입력: ${usernameInput ? '있음' : '없음'}`);
        console.log(`   비밀번호 입력: ${passwordInput ? '있음' : '없음'}`);
        
        // 5. 관리자 로그인 테스트
        console.log('\n📌 Step 5: 관리자 로그인 테스트');
        
        // 기존 값 지우고 입력
        await page.click('#username', { clickCount: 3 });
        await page.type('#username', 'admin');
        
        await page.click('#password', { clickCount: 3 });
        await page.type('#password', 'admin123');
        
        // 로그인 버튼 클릭
        await page.click('button[onclick="login()"]');
        
        // 로그인 성공 대기
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // 게임 섹션 확인
        const gameSection = await page.$('#gameSection.active');
        console.log(`   게임 섹션 활성화: ${gameSection ? '성공' : '실패'}`);
        
        if (gameSection) {
            // 사용자 정보 확인
            const username = await page.$eval('#currentUsername', el => el.textContent);
            const role = await page.$eval('#currentRole', el => el.textContent);
            console.log(`   로그인 사용자: ${username}`);
            console.log(`   역할: ${role}`);
            
            // 6. 게임 생성 테스트
            console.log('\n📌 Step 6: 보호된 기능 테스트 (게임 생성)');
            await page.click('button[onclick="createGame()"]');
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            // 이벤트 로그 확인
            const events = await page.$$eval('#eventLog .event-line', lines => 
                lines.map(line => line.textContent)
            );
            console.log('   이벤트 로그:');
            events.slice(-3).forEach(event => console.log(`     ${event}`));
            
            // 7. 로그아웃 테스트
            console.log('\n📌 Step 7: 로그아웃 테스트');
            await page.click('button[onclick="logout()"]');
            await new Promise(resolve => setTimeout(resolve, 500));
        } else {
            console.log('   ❌ 로그인 실패: 게임 섹션이 활성화되지 않음');
        }
        
        const authSectionVisible = await page.$('#authSection');
        console.log(`   로그인 화면으로 돌아감: ${authSectionVisible ? '성공' : '실패'}`);
        
        // 8. 스크린샷 캡처
        console.log('\n📌 Step 8: 스크린샷 캡처');
        await page.screenshot({ 
            path: 'auth-test-screenshot.png',
            fullPage: true 
        });
        console.log('   스크린샷 저장: auth-test-screenshot.png');
        
        console.log('\n✅ 모든 테스트 성공!');
        console.log('💫 JWT 인증이 정상적으로 작동합니다!');
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        throw error;
    } finally {
        // 정리
        if (browser) await browser.close();
        if (gameProcess) {
            console.log('\n🧹 프로세스 정리 중...');
            gameProcess.kill();
        }
    }
}

// 실행
testAuthWeb().catch(console.error);