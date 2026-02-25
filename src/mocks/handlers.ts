import { http, HttpResponse } from 'msw'

type Word = {
    word: string
    explain: string
    example: string
    add_time: number
    update_time: number
    reminder_time: number
    anki_count: number
    priority: number
    type: number
}

let words: Word[] = [
  {
    word: "hello",
    explain: "Greeting",
    example: "Hello world",
    add_time: Date.now() / 1000,
    update_time: Date.now() / 1000,
    reminder_time: Date.now() / 1000,
    anki_count: 0,
    priority: 0,
    type: 0
  },
   {
    word: "world",
    explain: "The earth, together with all of its countries, peoples, and natural features.",
    example: "He wants to see the world.",
    add_time: Date.now() / 1000,
    update_time: Date.now() / 1000,
    reminder_time: Date.now() / 1000,
    anki_count: 0,
    priority: 1,
    type: 0
  },
  {
    word: "事柄",
    explain: `[ことがら][kotogara]④或◎
<audio controls controlsList="nodownload" preload="none" src="http://d1.g.hjfile.cn/voice/jpsound/J26521.mp3"></audio>

- simple explain
  - 【名词】
    - 事情，事体；事态。

- More Detail
  - 名词
    - ものごとの模様・ありさま・内容。事情。情况。事态。
      - 見てきた事柄。
        亲眼看到的情况。
      - いかがわしい事柄。
        可疑的事情。 试试。`,
    example: "事柄の性質上",
    add_time: Date.now() / 1000,
    update_time: Date.now() / 1000,
    reminder_time: Date.now() / 1000,
    anki_count: 0,
    priority: 1,
    type: 0
  }
]

export const handlers = [
  http.post('/word/list', async ({ request }) => {
    const { page_size, page_number, order_by, type } = await request.json() as { page_size: number; page_number: number; order_by: string; type: number; }
    // Simple filter
    const filtered = words.filter(w => w.type === type)

    // Simple sort (mocking basic sorting)
    if (order_by === 'word') filtered.sort((a, b) => a.word.localeCompare(b.word))
    else if (order_by === 'word desc') filtered.sort((a, b) => b.word.localeCompare(a.word))
    else if (order_by === 'priority') filtered.sort((a, b) => a.priority - b.priority)
    else if (order_by === 'priority desc') filtered.sort((a, b) => b.priority - a.priority)

    const start = (page_number - 1) * page_size
    const end = start + page_size
    return HttpResponse.json(filtered.slice(start, end))
  }),

  http.post('/word/count', async ({ request }) => {
    const { type } = await request.json() as any
    const count = words.filter(w => w.type === type).length
    return HttpResponse.json({ size: count })
  }),

  http.post('/word/save', async ({ request }) => {
    const { origin, word, explain, example, type } = await request.json() as any
    const now = Date.now() / 1000

    if (origin) {
        // Edit
        const idx = words.findIndex(w => w.word === origin)
        if (idx !== -1) {
            words[idx] = { ...words[idx], word, explain, example, type, update_time: now }
        }
    } else {
        // Add
        words.unshift({ // Add to top
            word,
            explain,
            example,
            type,
            priority: 0,
            anki_count: 0,
            add_time: now,
            update_time: now,
            reminder_time: now
        })
    }
    return HttpResponse.json({})
  }),

  http.post('/word/delete', async ({ request }) => {
    const { word } = await request.json() as any
    words = words.filter(w => w.word !== word)
    return HttpResponse.json({})
  }),

  http.post('/word/remind_count_increment', async ({ request }) => {
    const { word } = await request.json() as any
    const w = words.find(w => w.word === word)
    if (w) w.anki_count++
    return HttpResponse.json({})
  }),

  http.post('/word/priority', async ({ request }) => {
    const { word, priority } = await request.json() as any
    const w = words.find(w => w.word === word)
    if (w) w.priority = priority
    return HttpResponse.json({})
  }),

  http.post('/word/ai_custom', () => {
    return HttpResponse.json([
        { name: "Mock LLM", models: ["gpt-4-mock", "claude-mock"] }
    ])
  }),

  http.post('/word/query', async ({ request }) => {
     const { word } = await request.json() as any
     // Simulate delay
     await new Promise(resolve => setTimeout(resolve, 500));
     return HttpResponse.json({
         result: `Mock translation for: ${word}\n\nThis is a mock response from the MSW handler.`,
         reasoning: `Mock reasoning for: ${word}`
     })
  }),

  http.post('/login', async () => {
    await new Promise(resolve => setTimeout(resolve, 500));
    return HttpResponse.json({
        token: "mock_jwt_token_example"
    })
  })
]
