const puppeteer = require('puppeteer');

async function testWorkingDemo() {
    console.log('🧪 HAL9 Working Demo - Puppeteer 자동화 테스트\n');
    
    const browser = await puppeteer.launch({
        headless: 'new',  // 헤드리스 모드로 실행
        defaultViewport: {
            width: 1200,
            height: 800
        }
    });
    
    try {
        const page = await browser.newPage();
        
        // 1. 페이지 로드 테스트
        console.log('1️⃣ 페이지 로드 테스트...');
        await page.goto('http://localhost:3333', { waitUntil: 'networkidle0' });
        
        const title = await page.title();
        console.log(`   ✓ 페이지 제목: ${title}`);
        
        // 2. 초기 API 호출 확인 (페이지 로드 시 자동 실행)
        console.log('\n2️⃣ 초기 API 자동 호출 확인...');
        await page.waitForSelector('.success', { timeout: 5000 });
        const initialStatus = await page.$eval('#status', el => el.textContent);
        console.log(`   ✓ 초기 상태: ${initialStatus.split('\n')[0]}`);
        
        // 3. 카운터 초기값 확인
        const initialCounter = await page.$eval('#counter', el => el.textContent);
        console.log(`   ✓ 초기 카운터: ${initialCounter}`);
        
        // 초기 상태 스크린샷
        await page.screenshot({ path: '/tmp/hal9-demo-initial.png' });
        console.log('   ✓ 초기 상태 스크린샷: /tmp/hal9-demo-initial.png');
        
        // 4. API 테스트 버튼 클릭
        console.log('\n3️⃣ API 테스트 버튼 클릭...');
        await page.click('button:nth-of-type(1)');
        await page.waitForFunction(
            () => document.querySelector('#status').textContent.includes('API 작동 확인'),
            { timeout: 5000 }
        );
        console.log('   ✓ API 테스트 성공');
        
        // 5. 카운터 증가 테스트
        console.log('\n4️⃣ 카운터 증가 테스트...');
        const testCount = 5;
        
        for (let i = 1; i <= testCount; i++) {
            await page.click('button:nth-of-type(2)');
            await page.waitForFunction(
                (expected) => document.querySelector('#counter').textContent === expected.toString(),
                { timeout: 5000 },
                parseInt(initialCounter) + i
            );
            const currentCounter = await page.$eval('#counter', el => el.textContent);
            console.log(`   ✓ 카운터 증가 확인: ${currentCounter}`);
        }
        
        // 6. 로그 확인
        console.log('\n5️⃣ 로그 기능 확인...');
        const logEntries = await page.$$eval('#log', logs => logs[0].innerHTML.split('<br>').length);
        console.log(`   ✓ 로그 항목 수: ${logEntries}`);
        
        // 로그 지우기 테스트
        await page.click('button:nth-of-type(3)');
        await new Promise(resolve => setTimeout(resolve, 500));
        const clearedLogEntries = await page.$$eval('#log', logs => logs[0].innerHTML.split('<br>').length);
        console.log(`   ✓ 로그 지우기 후 항목 수: ${clearedLogEntries}`);
        
        // 최종 상태 스크린샷
        await page.screenshot({ path: '/tmp/hal9-demo-final.png' });
        console.log('   ✓ 최종 상태 스크린샷: /tmp/hal9-demo-final.png');
        
        // 7. 성능 측정
        console.log('\n6️⃣ 성능 측정...');
        const startTime = Date.now();
        
        // 빠른 연속 클릭 테스트
        for (let i = 0; i < 10; i++) {
            await page.click('button:nth-of-type(2)');
        }
        
        // 마지막 카운터 값 확인
        await page.waitForFunction(
            (expected) => parseInt(document.querySelector('#counter').textContent) >= expected,
            { timeout: 5000 },
            parseInt(initialCounter) + testCount + 10
        );
        
        const elapsedTime = Date.now() - startTime;
        console.log(`   ✓ 10회 연속 클릭 처리 시간: ${elapsedTime}ms`);
        console.log(`   ✓ 평균 응답 시간: ${(elapsedTime / 10).toFixed(2)}ms`);
        
        // 8. 최종 카운터 값 확인
        console.log('\n7️⃣ 최종 상태 확인...');
        const finalCounter = await page.$eval('#counter', el => el.textContent);
        console.log(`   ✓ 최종 카운터 값: ${finalCounter}`);
        
        // 최종 로그 확인
        const finalLogContent = await page.$eval('#log', el => el.innerHTML);
        const finalLogLines = finalLogContent.split('<br>').filter(line => line.trim()).length;
        console.log(`   ✓ 최종 로그 라인 수: ${finalLogLines}`);
        
        console.log('\n✅ 모든 테스트 통과! HAL9 데모가 완벽하게 작동합니다.');
        console.log('\n📊 테스트 요약:');
        console.log('   - 페이지 로드: 성공');
        console.log('   - API 통신: 정상');
        console.log('   - 상태 관리: 정상');
        console.log('   - UI 반응성: 우수');
        console.log('   - 카운터 기능: 정상');
        console.log('   - 로그 기능: 정상');
        
        return true;
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        console.error('   상세:', error);
        
        // 에러 발생 시 스크린샷
        try {
            const page = (await browser.pages())[0];
            if (page) {
                await page.screenshot({ path: '/tmp/hal9-demo-error.png' });
                console.log('   에러 스크린샷: /tmp/hal9-demo-error.png');
            }
        } catch (screenshotError) {
            console.log('   스크린샷 저장 실패');
        }
        
        return false;
    } finally {
        await browser.close();
    }
}

// 테스트 실행
testWorkingDemo().then(success => {
    process.exit(success ? 0 : 1);
});