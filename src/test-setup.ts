// Node 25 ships a built-in localStorage that requires --localstorage-file to
// fully function. When that flag is absent or invalid, localStorage exists as
// an object but its methods (clear, setItem, etc.) are not callable.
// jsdom provides a real in-memory implementation; install it as the global so
// tests that call localStorage.clear() / setItem() / getItem() work correctly.
import { JSDOM } from 'jsdom'

const dom = new JSDOM('', { url: 'http://localhost/' })

// Replace the node built-in with jsdom's full implementation.
Object.defineProperty(globalThis, 'localStorage', {
  value: dom.window.localStorage,
  writable: true,
  configurable: true,
})
