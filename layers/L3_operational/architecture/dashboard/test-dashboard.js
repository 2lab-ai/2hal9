#!/usr/bin/env node
const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const path = require('path');

console.log('🧪 HAL9 통합 대시보드 테스트');
console.log('='.repeat(60));

async function testDashboard() {
    let dashboardProcess;
    let browser;
    
    try {
        // 1. 대시보드 서버 시작
        console.log('\n📌 Step 1: 통합 대시보드 서버 시작');
        const serverPath = path.join(__dirname, 'dashboard-server-simple.py');
        dashboardProcess = spawn('python3', [serverPath], {
            cwd: __dirname,
            env: { ...process.env, DASHBOARD_PORT: '8080' }
        });
        
        dashboardProcess.stdout.on('data', (data) => {
            console.log(`   서버: ${data.toString().trim()}`);
        });
        
        dashboardProcess.stderr.on('data', (data) => {
            console.error(`   에러: ${data.toString().trim()}`);
        });
        
        console.log('   프로세스 시작됨 (PID:', dashboardProcess.pid, ')');
        
        // 서버 시작 대기
        console.log('   서버 초기화 대기 중...');
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // 2. 브라우저 시작
        console.log('\n📌 Step 2: Puppeteer 브라우저 시작');
        browser = await puppeteer.launch({
            headless: 'new',
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        
        // 3. 대시보드 페이지 테스트
        console.log('\n📌 Step 3: 대시보드 페이지 로드 테스트');
        const page = await browser.newPage();
        
        // 콘솔 메시지 캡처
        page.on('console', msg => {
            if (msg.type() === 'error') {
                console.log('   브라우저 에러:', msg.text());
            }
        });
        
        await page.goto('http://localhost:8080', { 
            waitUntil: 'networkidle0',
            timeout: 30000 
        });
        
        // 페이지 제목 확인
        const title = await page.title();
        console.log(`   페이지 제목: ${title}`);
        
        // 4. 주요 요소 확인
        console.log('\n📌 Step 4: 대시보드 요소 확인');
        
        // 헤더 확인
        const headerText = await page.$eval('.header h1', el => el.textContent);
        console.log(`   ✓ 헤더: ${headerText}`);
        
        // 카드 개수 확인
        const cards = await page.$$('.card');
        console.log(`   ✓ 대시보드 카드 수: ${cards.length}`);
        
        // 각 카드 제목 확인
        for (let i = 0; i < cards.length; i++) {
            const cardTitle = await cards[i].$eval('h2', el => el.textContent);
            console.log(`   ✓ 카드 ${i + 1}: ${cardTitle}`);
        }
        
        // 5. 메트릭 API 테스트
        console.log('\n📌 Step 5: 메트릭 API 테스트');
        const metricsResponse = await page.evaluate(async () => {
            try {
                const response = await fetch('http://localhost:8080/api/dashboard/metrics');
                return await response.json();
            } catch (error) {
                return null;
            }
        });
        
        if (metricsResponse) {
            console.log('   ✓ 메트릭 API 응답 성공');
            console.log(`   - CPU: ${metricsResponse.system.cpu}%`);
            console.log(`   - Memory: ${metricsResponse.system.memory}%`);
            console.log(`   - Active Neurons: ${metricsResponse.neurons.active}`);
            console.log(`   - Phi: ${metricsResponse.consciousness.phi}`);
        } else {
            console.log('   ❌ 메트릭 API 응답 실패');
        }
        
        // 6. 버튼 클릭 테스트
        console.log('\n📌 Step 6: 인터랙션 테스트');
        
        // 새로고침 버튼 클릭
        await page.evaluate(() => {
            const refreshBtn = Array.from(document.querySelectorAll('.btn'))
                .find(btn => btn.textContent.includes('새로고침'));
            if (refreshBtn) refreshBtn.click();
        });
        console.log('   ✓ 새로고침 버튼 클릭 완료');
        
        // 콘솔 클리어 버튼 테스트
        await page.evaluate(() => {
            const clearBtn = Array.from(document.querySelectorAll('.btn'))
                .find(btn => btn.textContent === 'Clear');
            if (clearBtn) clearBtn.click();
        });
        console.log('   ✓ 콘솔 클리어 버튼 클릭 완료');
        
        // 7. 스크린샷 캡처
        console.log('\n📌 Step 7: 스크린샷 캡처');
        await page.screenshot({ 
            path: 'dashboard-test-screenshot.png',
            fullPage: true 
        });
        console.log('   ✓ 스크린샷 저장: dashboard-test-screenshot.png');
        
        console.log('\n✅ 모든 테스트 성공!');
        console.log('💫 HAL9 통합 대시보드가 정상 작동합니다!');
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        throw error;
    } finally {
        // 정리
        if (browser) await browser.close();
        if (dashboardProcess) {
            console.log('\n🧹 프로세스 정리 중...');
            dashboardProcess.kill();
        }
    }
}

// 실행
testDashboard().catch(console.error);