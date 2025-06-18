const puppeteer = require('puppeteer');

async function testDemo() {
    console.log('🧪 Puppeteer 자동화 테스트 시작...\n');
    
    const browser = await puppeteer.launch({
        headless: 'new', // 헤드리스 모드로 실행
        slowMo: 0        // 빠르게 실행
    });
    
    try {
        const page = await browser.newPage();
        
        // 1. 페이지 로드 테스트
        console.log('1️⃣ 페이지 로드 테스트...');
        await page.goto('http://localhost:8888', { waitUntil: 'networkidle0' });
        
        const title = await page.title();
        console.log(`   ✓ 페이지 제목: ${title}`);
        
        // 2. 요소 존재 확인
        console.log('\n2️⃣ UI 요소 확인...');
        const button = await page.$('button');
        if (button) {
            console.log('   ✓ 테스트 버튼 존재');
        } else {
            throw new Error('버튼을 찾을 수 없습니다');
        }
        
        // 3. 버튼 클릭 전 스크린샷
        await page.screenshot({ path: '/tmp/before-click.png' });
        console.log('   ✓ 클릭 전 스크린샷 저장: /tmp/before-click.png');
        
        // 4. 버튼 클릭
        console.log('\n3️⃣ 버튼 클릭 테스트...');
        await page.click('button');
        
        // API 응답 대기
        await page.waitForFunction(
            () => document.querySelector('#result').textContent.includes('서버 응답'),
            { timeout: 5000 }
        );
        
        // 5. 결과 확인
        const result = await page.$eval('#result', el => el.textContent);
        console.log('   ✓ 서버 응답 수신:');
        console.log(`     ${result.replace(/\n/g, '\n     ')}`);
        
        // 6. 클릭 후 스크린샷
        await page.screenshot({ path: '/tmp/after-click.png' });
        console.log('\n   ✓ 클릭 후 스크린샷 저장: /tmp/after-click.png');
        
        // 7. 성공 확인
        if (result.includes('작동 중')) {
            console.log('\n✅ 모든 테스트 통과! 데모가 정상 작동합니다.');
            return true;
        } else {
            throw new Error('예상된 응답을 받지 못했습니다');
        }
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        return false;
    } finally {
        await browser.close();
    }
}

// 테스트 실행
testDemo().then(success => {
    process.exit(success ? 0 : 1);
});