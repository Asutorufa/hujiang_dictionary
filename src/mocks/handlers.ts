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
  }
]

export const handlers = [
  http.post('/word/list', async ({ request }) => {
    const { page_size, page_number, order_by, type } = await request.json() as { page_size: number; page_number: number; order_by: string; type: number; }
    // Simple filter
    let filtered = words.filter(w => type === undefined || type === 0 || w.type === type)
    if (type === 1) { // Grammar
         filtered = words.filter(w => w.type === 1)
    } else { // Word (0) or all? Code sends 0 for word, 1 for grammar.
         // app logic: grammar ? 1 : 0.
         // If type is 0, it likely means words.
         filtered = words.filter(w => w.type === type)
    }

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
  })
]
