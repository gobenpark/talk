# 실시간 날씨 에이전트 데모

이 예제는 실제 OpenWeatherMap API를 사용하여 날씨 정보를 가져오는 AI 에이전트를 보여줍니다.

## 준비 사항

### 1. OpenWeatherMap API 키 받기

1. https://openweathermap.org/api 방문
2. 무료 계정 생성
3. API 키 받기 (무료 tier는 하루 1,000회 호출 가능)

### 2. OpenAI API 키 받기

1. https://platform.openai.com/ 방문
2. API 키 생성

## 실행 방법

### 환경 변수 설정

```bash
# OpenWeather API 키 설정
export OPENWEATHER_API_KEY=your_openweather_api_key_here

# OpenAI API 키 설정
export OPENAI_API_KEY=your_openai_api_key_here
```

### 예제 실행

```bash
# 실시간 날씨 API 사용
cargo run --example weather_agent_live

# Mock 데이터 사용 (API 키 불필요)
cargo run --example weather_agent
```

## 동작 방식

1. **사용자 입력**: "What's the weather in Seoul?"
2. **가이드라인 매칭**: regex 패턴으로 날씨 질문 감지
3. **도구 실행**: OpenWeatherMap API 호출
4. **LLM 응답**: OpenAI GPT가 날씨 데이터를 자연스러운 문장으로 변환
5. **사용자에게 응답**: "Seoul is currently 15°C with clear skies..."

## 지원되는 질문 패턴

- "What's the weather in [도시]?"
- "How's the weather in [도시]?"
- "Tell me about the weather in [도시]"
- "weather in [도시]" (regex 패턴 매칭)

## 예제 출력

```
🌤️  Live Weather Agent Example
================================

✅ API keys configured
✅ Live weather tool registered
✅ Weather guideline registered

📝 Created session: 01234567-89ab-cdef-0123-456789abcdef

👤 User: What's the weather in Seoul?
   🌐 Fetching weather data for: Seoul
   ✅ Weather data fetched successfully
🤖 Agent: The current weather in Seoul is 15.3°C (59.5°F) with clear sky.
   The humidity is at 45% and there's a light wind at 6.7 mph.

   🔧 Tools used: 1 tool(s)
      - Tool ID: get_weather (took 342ms)
   📊 Confidence: 90.00%
```

## 기술 스택

- **Rust**: 시스템 언어
- **Tokio**: 비동기 런타임
- **reqwest**: HTTP 클라이언트
- **OpenWeatherMap API**: 날씨 데이터
- **LLM**: OpenAI GPT-3.5 또는 Anthropic Claude Sonnet 4.5

## 특징

### ✅ 실제 API 통합
- OpenWeatherMap으로부터 실시간 날씨 데이터
- 섭씨/화씨 온도 변환
- 습도, 풍속 정보

### ✅ 에러 처리
- API 키 검증
- 도시명 오류 처리
- 네트워크 오류 처리
- 타임아웃 처리 (30초)

### ✅ 재시도 로직
- 지수 백오프를 사용한 자동 재시도
- 최대 3회 재시도
- 실패 시 명확한 에러 메시지

### ✅ 로깅
- 상세한 실행 로그
- 도구 실행 시간 측정
- 설명 가능한 AI 응답

## 다음 단계

1. **다른 도시 시도**: 전 세계 어떤 도시든 가능
2. **추가 기능**: 일기 예보, 공기 질, UV 지수 등
3. **다른 도구 추가**: 뉴스, 번역, 계산기 등
4. **대화 여정**: 다단계 대화 흐름 구현

## 문제 해결

### API 키 오류
```
❌ OPENWEATHER_API_KEY not set!
```
→ 환경 변수가 제대로 설정되었는지 확인

### 도시를 찾을 수 없음
```
Weather API returned error: 404 - City 'XYZ' not found
```
→ 올바른 도시명 사용 (영어로)

### 네트워크 오류
```
Failed to fetch weather data: connection timeout
```
→ 인터넷 연결 확인

## 라이센스

MIT OR Apache-2.0
