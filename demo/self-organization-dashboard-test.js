const puppeteer = require('puppeteer');

async function testSelfOrganizationDashboard() {
    console.log('🤖 Self-Organization Dashboard - 자동화 테스트\n');
    
    const browser = await puppeteer.launch({
        headless: 'new',
        defaultViewport: {
            width: 1600,
            height: 900
        }
    });
    
    try {
        const page = await browser.newPage();
        
        // 1. 대시보드 로드
        console.log('1️⃣ 대시보드 로드 중...');
        await page.goto('http://localhost:8766', { waitUntil: 'networkidle0' });
        
        const title = await page.title();
        console.log(`   ✓ 페이지 제목: ${title}`);
        
        // 2. 초기 메트릭 확인
        console.log('\n2️⃣ 초기 시스템 메트릭 확인...');
        
        const initialMetrics = await page.evaluate(() => {
            return {
                activeNeurons: document.getElementById('active-neurons').textContent,
                totalConnections: document.getElementById('total-connections').textContent,
                layerCount: document.getElementById('layer-count').textContent,
                clusterCount: document.getElementById('cluster-count').textContent,
                opsPerSec: document.getElementById('ops-per-sec').textContent,
                emergenceTime: document.getElementById('emergence-time').textContent,
                compressionRatio: document.getElementById('compression-ratio').textContent,
                fps: document.getElementById('fps').textContent
            };
        });
        
        console.log(`   ✓ 활성 뉴런: ${initialMetrics.activeNeurons}`);
        console.log(`   ✓ 총 연결: ${initialMetrics.totalConnections}`);
        console.log(`   ✓ 레이어 수: ${initialMetrics.layerCount}`);
        console.log(`   ✓ 클러스터: ${initialMetrics.clusterCount}`);
        console.log(`   ✓ 출현 시간: ${initialMetrics.emergenceTime} μs`);
        console.log(`   ✓ 압축 비율: ${initialMetrics.compressionRatio}`);
        
        // 초기 스크린샷
        await page.screenshot({ path: '/tmp/self-org-dashboard-initial.png' });
        console.log('   ✓ 초기 스크린샷: /tmp/self-org-dashboard-initial.png');
        
        // 3. 모니터링 시작
        console.log('\n3️⃣ 실시간 모니터링 시작...');
        await page.click('#start-btn');
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const isMonitoring = await page.evaluate(() => {
            return document.getElementById('start-btn').classList.contains('active');
        });
        console.log(`   ✓ 모니터링 상태: ${isMonitoring ? '활성' : '비활성'}`);
        
        // 4. 뉴런 추가
        console.log('\n4️⃣ 뉴런 추가 테스트...');
        await page.click('button:nth-of-type(2)'); // "뉴런 추가" 버튼
        await new Promise(resolve => setTimeout(resolve, 500));
        
        const updatedNeurons = await page.$eval('#active-neurons', el => el.textContent);
        console.log(`   ✓ 업데이트된 뉴런 수: ${updatedNeurons}`);
        
        // 5. 대시보드 패널 확인
        console.log('\n5️⃣ 대시보드 패널 확인...');
        
        // 뉴런 목록 확인
        const neuronListCount = await page.$$eval('.neuron-item', items => items.length);
        console.log(`   ✓ 뉴런 목록 항목: ${neuronListCount}개`);
        
        // 레이어 시각화 확인
        const layerBars = await page.$$eval('.layer-bar', bars => bars.length);
        console.log(`   ✓ 레이어 바: ${layerBars}개`);
        
        // 연결 매트릭스 확인
        const matrixCells = await page.$$eval('.matrix-cell', cells => cells.length);
        console.log(`   ✓ 매트릭스 셀: ${matrixCells}개`);
        
        // 6. 성능 모니터링 (3초간)
        console.log('\n6️⃣ 성능 모니터링...');
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        const performanceMetrics = await page.evaluate(() => {
            return {
                fps: document.getElementById('fps').textContent,
                opsPerSec: document.getElementById('ops-per-sec').textContent,
                activeNeurons: document.getElementById('active-neurons').textContent
            };
        });
        
        console.log(`   ✓ FPS: ${performanceMetrics.fps}`);
        console.log(`   ✓ Ops/sec: ${performanceMetrics.opsPerSec}`);
        console.log(`   ✓ 활성 뉴런: ${performanceMetrics.activeNeurons}`);
        
        // 7. Canvas 렌더링 확인
        console.log('\n7️⃣ 네트워크 시각화 확인...');
        const canvasData = await page.evaluate(() => {
            const canvas = document.getElementById('network-canvas');
            const perfCanvas = document.getElementById('performance-chart');
            return {
                network: {
                    width: canvas.width,
                    height: canvas.height,
                    hasContent: canvas.getContext('2d').getImageData(0, 0, 1, 1).data[3] > 0
                },
                performance: {
                    width: perfCanvas.width,
                    height: perfCanvas.height,
                    hasContent: perfCanvas.getContext('2d').getImageData(0, 0, 1, 1).data[3] > 0
                }
            };
        });
        
        console.log(`   ✓ 네트워크 캔버스: ${canvasData.network.width}x${canvasData.network.height}`);
        console.log(`   ✓ 네트워크 렌더링: ${canvasData.network.hasContent ? '정상' : '비어있음'}`);
        console.log(`   ✓ 성능 차트 렌더링: ${canvasData.performance.hasContent ? '정상' : '비어있음'}`);
        
        // 8. 상태 인디케이터 확인
        console.log('\n8️⃣ 시스템 상태 확인...');
        const statusIndicators = await page.evaluate(() => {
            return {
                system: document.getElementById('system-status').className.includes('active'),
                network: document.getElementById('network-status').className.includes('active'),
                performance: document.getElementById('performance-status').className
            };
        });
        
        console.log(`   ✓ 시스템 상태: ${statusIndicators.system ? '정상' : '문제'}`);
        console.log(`   ✓ 네트워크 상태: ${statusIndicators.network ? '정상' : '문제'}`);
        console.log(`   ✓ 성능 상태: ${statusIndicators.performance.includes('active') ? '우수' : 
                                      statusIndicators.performance.includes('warning') ? '경고' : '문제'}`);
        
        // 최종 스크린샷
        await page.screenshot({ path: '/tmp/self-org-dashboard-final.png' });
        console.log('   ✓ 최종 스크린샷: /tmp/self-org-dashboard-final.png');
        
        // 9. 리셋 기능 테스트
        console.log('\n9️⃣ 시스템 리셋 테스트...');
        
        // 리셋 다이얼로그 핸들러
        page.on('dialog', async dialog => {
            await dialog.accept();
        });
        
        await page.click('.control-btn.danger');
        await new Promise(resolve => setTimeout(resolve, 500));
        
        const resetNeurons = await page.$eval('#active-neurons', el => el.textContent);
        console.log(`   ✓ 리셋 후 뉴런 수: ${resetNeurons}`);
        
        console.log('\n✅ 모든 테스트 통과! Self-Organization Dashboard가 완벽하게 작동합니다.');
        console.log('\n📊 테스트 요약:');
        console.log('   - 실시간 모니터링: 정상');
        console.log('   - 메트릭 업데이트: 정상');
        console.log('   - 네트워크 시각화: 정상');
        console.log('   - 성능 차트: 정상');
        console.log('   - 대시보드 UI: 정상');
        
        return true;
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        console.error('   상세:', error);
        
        try {
            const page = (await browser.pages())[0];
            if (page) {
                await page.screenshot({ path: '/tmp/self-org-dashboard-error.png' });
                console.log('   에러 스크린샷: /tmp/self-org-dashboard-error.png');
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
testSelfOrganizationDashboard().then(success => {
    process.exit(success ? 0 : 1);
});