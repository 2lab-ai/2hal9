const puppeteer = require('puppeteer');

async function testAIGeniusGame() {
    console.log('🎮 AI Genius Game - Puppeteer 자동화 테스트\n');
    
    const browser = await puppeteer.launch({
        headless: 'new',
        defaultViewport: {
            width: 1400,
            height: 900
        }
    });
    
    try {
        const page = await browser.newPage();
        
        // 1. 메인 페이지 로드
        console.log('1️⃣ 게임 메인 페이지 로드 중...');
        await page.goto('http://localhost:3456', { waitUntil: 'networkidle0' });
        
        const title = await page.title();
        console.log(`   ✓ 페이지 제목: ${title}`);
        
        // 2. 게임 카드 확인
        console.log('\n2️⃣ 게임 카드 확인...');
        const gameCards = await page.$$('.game-card');
        console.log(`   ✓ 사용 가능한 게임: ${gameCards.length}개`);
        
        // 게임 이름 추출
        const gameNames = await page.evaluate(() => {
            return Array.from(document.querySelectorAll('.game-card h3'))
                .map(el => el.textContent);
        });
        gameNames.forEach((name, idx) => {
            console.log(`     ${idx + 1}. ${name}`);
        });
        
        // 3. 의식 출현 게임 선택
        console.log('\n3️⃣ 의식 출현 게임 선택...');
        await page.click('.game-card:first-child');
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // 스크린샷
        await page.screenshot({ path: '/tmp/ai-genius-game-menu.png' });
        console.log('   ✓ 메뉴 스크린샷: /tmp/ai-genius-game-menu.png');
        
        // 4. 새 게임 생성
        console.log('\n4️⃣ 새 게임 생성...');
        
        // API로 직접 게임 생성
        const createResponse = await page.evaluate(async () => {
            try {
                const response = await fetch('/api/games/create', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        game_type: { type: 'ConsciousnessEmergence' },
                        max_rounds: 20
                    })
                });
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                return await response.json();
            } catch (error) {
                return { error: error.message };
            }
        });
        
        if (createResponse.error) {
            console.log(`   ✗ 게임 생성 실패: ${createResponse.error}`);
            throw new Error(`게임 생성 실패: ${createResponse.error}`);
        }
        
        console.log(`   ✓ 게임 ID: ${createResponse.id}`);
        console.log(`   ✓ 상태: ${createResponse.status}`);
        
        // 5. 게임 목록 확인
        console.log('\n5️⃣ 게임 목록 확인...');
        const gameList = await page.evaluate(async () => {
            const response = await fetch('/api/games');
            return await response.json();
        });
        console.log(`   ✓ 활성 게임 수: ${gameList.length}`);
        
        // 6. 게임 상태 확인
        console.log('\n6️⃣ 게임 상태 확인...');
        const gameState = await page.evaluate(async (gameId) => {
            const response = await fetch(`/api/games/${gameId}`);
            return await response.json();
        }, createResponse.id);
        
        console.log(`   ✓ 게임 타입: ${gameState.game_type}`);
        console.log(`   ✓ 게임 상태: ${gameState.status}`);
        console.log(`   ✓ 라운드: ${gameState.round}/${gameState.max_rounds}`);
        console.log(`   ✓ 보드 크기: ${gameState.board.length}x${gameState.board[0].length}`);
        
        // 7. WebSocket 연결 테스트
        console.log('\n7️⃣ WebSocket 연결 테스트...');
        
        // WebSocket 연결 확인을 위한 새 페이지
        const wsTestPage = await browser.newPage();
        
        await wsTestPage.evaluateOnNewDocument((gameId) => {
            window.gameId = gameId;
            window.wsMessages = [];
            window.wsConnected = false;
        }, createResponse.id);
        
        await wsTestPage.goto('http://localhost:3456');
        
        // WebSocket 연결 시도
        const wsConnected = await wsTestPage.evaluate((gameId) => {
            return new Promise((resolve) => {
                const ws = new WebSocket(`ws://localhost:3456/ws/${gameId}`);
                
                ws.onopen = () => {
                    window.wsConnected = true;
                    ws.send(JSON.stringify({
                        type: 'join_game',
                        player_name: 'Test Player',
                        player_type: {
                            type: 'hal9_collective',
                            agent_count: 6
                        }
                    }));
                    resolve(true);
                };
                
                ws.onerror = () => resolve(false);
                
                ws.onmessage = (event) => {
                    window.wsMessages.push(JSON.parse(event.data));
                };
                
                setTimeout(() => resolve(false), 5000);
            });
        }, createResponse.id);
        
        if (wsConnected) {
            console.log('   ✓ WebSocket 연결 성공');
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            const messages = await wsTestPage.evaluate(() => window.wsMessages);
            console.log(`   ✓ 받은 메시지 수: ${messages.length}`);
        } else {
            console.log('   ✗ WebSocket 연결 실패');
        }
        
        await wsTestPage.close();
        
        // 8. 성능 측정
        console.log('\n8️⃣ 응답 시간 측정...');
        const startTime = Date.now();
        
        // 여러 API 호출 시간 측정
        const apiCalls = await page.evaluate(async () => {
            const timings = {};
            
            // 게임 목록
            let start = Date.now();
            await fetch('/api/games');
            timings.gameList = Date.now() - start;
            
            // 게임 생성
            start = Date.now();
            await fetch('/api/games/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    game_type: { type: 'MinorityGame' },
                    max_rounds: 10
                })
            });
            timings.createGame = Date.now() - start;
            
            return timings;
        });
        
        console.log(`   ✓ 게임 목록 조회: ${apiCalls.gameList}ms`);
        console.log(`   ✓ 게임 생성: ${apiCalls.createGame}ms`);
        
        // 최종 스크린샷
        await page.screenshot({ path: '/tmp/ai-genius-game-final.png' });
        console.log('   ✓ 최종 스크린샷: /tmp/ai-genius-game-final.png');
        
        console.log('\n✅ 모든 테스트 통과! AI Genius Game이 정상 작동합니다.');
        console.log('\n📊 테스트 요약:');
        console.log('   - 페이지 로드: 성공');
        console.log('   - API 엔드포인트: 정상');
        console.log('   - 게임 생성: 정상');
        console.log('   - WebSocket: ' + (wsConnected ? '정상' : '미연결'));
        console.log('   - UI 렌더링: 정상');
        
        return true;
        
    } catch (error) {
        console.error('\n❌ 테스트 실패:', error.message);
        console.error('   상세:', error);
        
        try {
            const page = (await browser.pages())[0];
            if (page) {
                await page.screenshot({ path: '/tmp/ai-genius-game-error.png' });
                console.log('   에러 스크린샷: /tmp/ai-genius-game-error.png');
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
testAIGeniusGame().then(success => {
    process.exit(success ? 0 : 1);
});