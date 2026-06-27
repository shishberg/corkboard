import { defineStore } from 'pinia'
import { fetchFeeds as apiFetchFeeds } from '@/lib/deviceApi'

export interface Feed { id: string; name: string }

interface FeedsState {
  feeds: Feed[]
}

export const useFeedsStore = defineStore('feeds', {
  state: (): FeedsState => ({
    feeds: [{ id: 'family', name: 'Family' }],
  }),
  actions: {
    async loadFeeds() {
      const result = await apiFetchFeeds()
      if (result !== null) {
        this.feeds = result
      }
      // On null (device unreachable) leave the stub list untouched
    },
  },
})
