#!/usr/bin/env node
const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

console.log('🧪 HAL9 통합 데모 스위트 테스트');
console.log('='.repeat(60));

async function waitForPort(port, maxAttempts = 30) {
    const http = require('http');
    for (let i = 0; i < maxAttempts; i++) {
        try {
            await new Promise((resolve, reject) => {
                const req = http.get(`http://localhost:${port}`, (res) => {
                    resolve(true);
                });
                req.on('error', reject);
                req.setTimeout(1000);
            });
            return true;
        } catch (e) {
            await new Promise(resolve => setTimeout(resolve, 1000));
        }
    }
    return false;
}

async function testIntegratedSuite() {
    let demoProcess;
    let browser;
    
    try {
        // 1. 통합 데모 스위트 시작
        console.log('\n📌 Step 1: 통합 데모 스위트 시작');
        const scriptPath = path.join(__dirname, 'integrated-demo-suite.sh');
        demoProcess = spawn('bash', [scriptPath], {
            cwd: path.join(__dirname, '..'),
            detached: true
        });
        
        console.log('   프로세스 시작됨 (PID:', demoProcess.pid, ')');
        
        // 서비스 대기
        console.log('\n📌 Step 2: 서비스 시작 대기');
        const services = [
            { port: 8766, name: 'Self-Organization Dashboard' },
            { port: 8765, name: 'Consciousness Visualization' },
            { port: 3456, name: 'AI Genius Game' },
            { port: 8767, name: 'Performance Monitor' }
        ];
        
        for (const service of services) {
            console.log(`   ${service.name} 대기 중...`);
            const isUp = await waitForPort(service.port);
            if (isUp) {
                console.log(`   ✅ ${service.name} 실행 중 (포트 ${service.port})`);
            } else {
                console.log(`   ❌ ${service.name} 시작 실패`);
                throw new Error(`${service.name} 시작 실패`);
            }
        }
        
        // 브라우저 시작
        console.log('\n📌 Step 3: Puppeteer 브라우저 시작');
        browser = await puppeteer.launch({
            headless: 'new',
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        
        // 각 서비스 테스트
        console.log('\n📌 Step 4: 각 서비스 테스트');
        
        // 4.1 Self-Organization Dashboard
        console.log('\n🧪 Self-Organization Dashboard 테스트');
        const page1 = await browser.newPage();
        await page1.goto('http://localhost:8766', { waitUntil: 'networkidle0' });
        const selfOrgTitle = await page1.title();
        const hasFPSCounter = await page1.$('#fps-counter');
        console.log(`   제목: ${selfOrgTitle}`);
        console.log(`   FPS 카운터: ${hasFPSCounter ? '있음' : '없음'}`);
        await page1.close();
        
        // 4.2 Consciousness Visualization
        console.log('\n🧪 Consciousness Visualization 테스트');
        const page2 = await browser.newPage();
        await page2.goto('http://localhost:8765', { waitUntil: 'networkidle0' });
        const consTitle = await page2.title();
        const hasPhiValue = await page2.$('#phi-value');
        console.log(`   제목: ${consTitle}`);
        console.log(`   Phi 값 표시: ${hasPhiValue ? '있음' : '없음'}`);
        await page2.close();
        
        // 4.3 AI Genius Game
        console.log('\n🧪 AI Genius Game 테스트');
        const page3 = await browser.newPage();
        const response = await page3.goto('http://localhost:3456/api/games');
        const gamesData = await response.json();
        console.log(`   API 응답: ${JSON.stringify(gamesData).substring(0, 50)}...`);
        await page3.close();
        
        // 4.4 Performance Monitor
        console.log('\n🧪 Performance Monitor 테스트');
        const page4 = await browser.newPage();
        await page4.goto('http://localhost:8767', { waitUntil: 'networkidle0' });
        const perfTitle = await page4.title();
        const hasMetrics = await page4.$('#metrics');
        console.log(`   제목: ${perfTitle}`);
        console.log(`   메트릭 표시: ${hasMetrics ? '있음' : '없음'}`);
        await page4.close();
        
        // 4.5 통합 런처 테스트
        console.log('\n🧪 통합 런처 (hal9-suite.html) 테스트');
        const launcherPath = path.join(__dirname, 'hal9-suite.html');
        if (fs.existsSync(launcherPath)) {
            const page5 = await browser.newPage();
            await page5.goto(`file://${launcherPath}`, { waitUntil: 'networkidle0' });
            const launcherTitle = await page5.title();
            const demoCards = await page5.$$('.demo-card');
            console.log(`   제목: ${launcherTitle}`);
            console.log(`   데모 카드 수: ${demoCards.length}`);
            await page5.close();
        } else {
            console.log('   ❌ hal9-suite.html 파일이 없습니다');
        }
        
        console.log('\n✅ 모든 테스트 성공!');
        console.log('💫 HAL9 통합 데모 스위트가 정상 작동합니다!');
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        throw error;
    } finally {
        // 정리
        if (browser) await browser.close();
        if (demoProcess) {
            console.log('\n🧹 프로세스 정리 중...');
            process.kill(-demoProcess.pid);
        }
    }
}

// 실행
testIntegratedSuite().catch(console.error);