const EXT_ID = 'paidtasks'

window.app = Vue.createApp({
  el: '#vue',
  mixins: [window.windowMixin],
  data() {
    return {
      loading: false,
      lists: [],
      tasks: [],
      paidMap: {},
      paidSockets: {},
      listForm: {
        name: '',
        description: '',
        wallet_id: null
      },
      taskForm: {
        title: '',
        list_id: null,
        cost_sats: 100
      },
      editListDialog: false,
      editTaskDialog: false,
      editList: null,
      editTask: null,
      suppressWalletSave: false,
      walletInitialized: false
    }
  },
  computed: {
    walletKey() {
      const wallets = this.g?.user?.wallets || []
      return wallets[0]?.inkey || null
    },
    walletOptions() {
      const wallets = this.g?.user?.wallets || []
      return wallets.map(w => ({label: w.name, value: w.id, inkey: w.inkey}))
    },
    listOptions() {
      return this.lists.map(l => ({label: l.name, value: l.id}))
    },
    listById() {
      const map = {}
      for (const list of this.lists) map[list.id] = list
      return map
    },
    walletById() {
      const map = {}
      for (const wallet of this.walletOptions) map[wallet.value] = wallet
      return map
    },
    listColumns() {
      return [
        {name: 'name', label: 'Name', field: 'name', align: 'left'},
        {name: 'description', label: 'Description', field: 'description', align: 'left'},
        {name: 'wallet', label: 'Wallet', field: 'wallet_id', align: 'left'},
        {name: 'public', label: 'Public', field: 'id', align: 'right'}
      ]
    },
    taskColumns() {
      return [
        {name: 'title', label: 'Task', field: 'title', align: 'left'},
        {name: 'list', label: 'List', field: 'list_id', align: 'left'},
        {name: 'cost', label: 'Cost (sats)', field: 'cost_sats', align: 'right'},
        {name: 'paid', label: 'Paid', field: 'id', align: 'right'}
      ]
    }
  },
  methods: {
    makeId() {
      const base = Math.floor(Date.now() / 1000)
      const extra = Math.floor(Math.random() * 1000)
      return base * 1000 + extra
    },
    isSafeId(value) {
      const num = Number(value)
      return Number.isInteger(num) && num > 0 && num < 2147483647
    },
    async remapUnsafeIds() {
      const listIdMap = new Map()
      let listsChanged = false
      let tasksChanged = false

      for (const list of this.lists) {
        if (!this.isSafeId(list.id)) {
          const newId = this.makeId()
          listIdMap.set(list.id, newId)
          list.id = newId
          listsChanged = true

          if (list.wallet_id) {
            const wallet = this.walletOptions.find(w => w.value === list.wallet_id)
            if (wallet) {
              await this.secretSet(`list_wallet_inkey:${newId}`, wallet.inkey)
            }
          }
          await this.secretDelete(`list_wallet_inkey:${list.id}`)
        }
      }

      for (const task of this.tasks) {
        if (listIdMap.has(task.list_id)) {
          task.list_id = listIdMap.get(task.list_id)
          tasksChanged = true
        }
        if (!this.isSafeId(task.id)) {
          const newId = this.makeId()
          const oldId = task.id
          task.id = newId
          tasksChanged = true
          await this.kvSet(`task_cost:${newId}`, String(task.cost_sats))
          await this.kvSet(`task_list:${newId}`, String(task.list_id))
          await Promise.all([
            this.kvSet(`task_cost:${oldId}`, ''),
            this.kvSet(`task_list:${oldId}`, ''),
            this.kvSet(`task_paid:${oldId}`, '')
          ])
        }
      }

      if (listsChanged) {
        await this.saveLists()
      }
      if (tasksChanged) {
        await this.saveTasks()
      }
    },
    publicListUrl(listId) {
      return `${window.location.origin}/${EXT_ID}/public/${listId}`
    },
    parseJsonValue(value, fallback) {
      if (value === null || value === undefined || value === '') return fallback
      try {
        return JSON.parse(value)
      } catch (err) {
        return fallback
      }
    },
    async kvGet(key) {
      const {data} = await LNbits.api.request(
        'GET',
        `/${EXT_ID}/api/v1/kv/${key}`,
        this.walletKey
      )
      return data?.value ?? null
    },
    async kvSet(key, value) {
      await LNbits.api.request(
        'POST',
        `/${EXT_ID}/api/v1/kv/${key}`,
        this.walletKey,
        {value}
      )
    },
    async secretSet(key, value) {
      await LNbits.api.request(
        'POST',
        `/${EXT_ID}/api/v1/secret/${key}`,
        this.walletKey,
        {value}
      )
    },
    async secretDelete(key) {
      await LNbits.api.request(
        'DELETE',
        `/${EXT_ID}/api/v1/secret/${key}`,
        this.walletKey
      )
    },
    async loadListsAndTasks() {
      const [listsValue, tasksValue] = await Promise.all([
        this.kvGet('lists'),
        this.kvGet('tasks')
      ])
      this.lists = this.parseJsonValue(listsValue, [])
      this.tasks = this.parseJsonValue(tasksValue, [])
      await this.remapUnsafeIds()
      await this.ensureTaskKeys()
      await this.ensureListSecrets()
      await this.saveLists()
      await this.saveTasks()
    },
    async ensureTaskKeys() {
      await Promise.all(
        this.tasks.map(task =>
          Promise.all([
            this.kvSet(`task_cost:${task.id}`, String(task.cost_sats)),
            this.kvSet(`task_list:${task.id}`, String(task.list_id))
          ])
        )
      )
    },
    async ensureListSecrets() {
      await Promise.all(
        this.lists.map(async list => {
          if (!list.wallet_id) return
          const wallet = this.walletOptions.find(w => w.value === list.wallet_id)
          if (!wallet) return
          await this.secretSet(`list_wallet_inkey:${list.id}`, wallet.inkey)
        })
      )
    },
    async loadPaidStatuses() {
      const paidMap = {}
      await Promise.all(
        this.tasks.map(async task => {
          const value = await this.kvGet(`task_paid:${task.id}`)
          paidMap[task.id] = value !== null
        })
      )
      this.paidMap = paidMap
    },
    connectPaidSockets() {
      Object.values(this.paidSockets).forEach(ws => {
        try {
          ws.close()
        } catch {}
      })
      this.paidSockets = {}
      this.tasks.forEach(task => {
        const url = new URL(window.location)
        url.protocol = url.protocol === 'https:' ? 'wss' : 'ws'
        url.pathname = `/api/v1/ws/${EXT_ID}:task_paid:${task.id}`
        const ws = new WebSocket(url)
        this.paidSockets[task.id] = ws
        ws.addEventListener('message', () => {
          this.paidMap = {...this.paidMap, [task.id]: true}
        })
        ws.addEventListener('error', () => {
          try {
            ws.close()
          } catch {}
        })
      })
    },
    async saveLists() {
      await Promise.all([
        this.kvSet('lists', JSON.stringify(this.lists)),
        this.kvSet('public_lists', JSON.stringify(this.lists))
      ])
    },
    async saveTasks() {
      await Promise.all([
        this.kvSet('tasks', JSON.stringify(this.tasks)),
        this.kvSet('public_tasks', JSON.stringify(this.tasks))
      ])
    },
    async createList() {
      if (!this.listForm.name || !this.listForm.wallet_id) return
      const newList = {
        id: this.makeId(),
        name: this.listForm.name.trim(),
        description: this.listForm.description.trim(),
        created_at: new Date().toISOString(),
        wallet_id: this.listForm.wallet_id
      }
      this.lists = [newList, ...this.lists]
      this.listForm.name = ''
      this.listForm.description = ''
      this.listForm.wallet_id = null
      const wallet = this.walletOptions.find(w => w.value === newList.wallet_id)
      if (wallet) {
        await this.secretSet(`list_wallet_inkey:${newList.id}`, wallet.inkey)
      }
      await this.saveLists()
    },
    openEditList(list) {
      this.editList = {...list}
      this.editListDialog = true
    },
    async saveEditList() {
      if (!this.editList || !this.editList.name || !this.editList.wallet_id) return
      const idx = this.lists.findIndex(l => l.id === this.editList.id)
      if (idx !== -1) {
        this.lists.splice(idx, 1, {
          ...this.lists[idx],
          name: this.editList.name.trim(),
          description: (this.editList.description || '').trim(),
          wallet_id: this.editList.wallet_id
        })
        const wallet = this.walletOptions.find(w => w.value === this.lists[idx].wallet_id)
        if (wallet) {
          await this.secretSet(`list_wallet_inkey:${this.lists[idx].id}`, wallet.inkey)
        }
        await this.saveLists()
      }
      this.editListDialog = false
      this.editList = null
    },
    async deleteList(list) {
      const ok = await this.$q.dialog({
        title: 'Delete list',
        message: 'Delete this list and its tasks?',
        cancel: true,
        persistent: true
      })
      if (!ok) return
      this.lists = this.lists.filter(l => l.id !== list.id)
      const removedTasks = this.tasks.filter(t => t.list_id === list.id)
      this.tasks = this.tasks.filter(t => t.list_id !== list.id)
      await this.saveLists()
      await this.saveTasks()
      await this.secretDelete(`list_wallet_inkey:${list.id}`)
      await Promise.all(
        removedTasks.map(t => Promise.all([
          this.kvSet(`task_cost:${t.id}`, ''),
          this.kvSet(`task_list:${t.id}`, ''),
          this.kvSet(`task_paid:${t.id}`, '')
        ]))
      )
      await this.loadPaidStatuses()
    },
    async createTask() {
      if (!this.taskForm.title || !this.taskForm.list_id || !this.taskForm.cost_sats) return
      const newTask = {
        id: this.makeId(),
        list_id: this.taskForm.list_id,
        title: this.taskForm.title.trim(),
        cost_sats: Number(this.taskForm.cost_sats),
        created_at: new Date().toISOString()
      }
      this.tasks = [newTask, ...this.tasks]
      this.taskForm.title = ''
      this.taskForm.cost_sats = 100
      await this.kvSet(`task_cost:${newTask.id}`, String(newTask.cost_sats))
      await this.kvSet(`task_list:${newTask.id}`, String(newTask.list_id))
      await this.saveTasks()
    },
    openEditTask(task) {
      this.editTask = {...task}
      this.editTaskDialog = true
    },
    async saveEditTask() {
      if (!this.editTask || !this.editTask.title || !this.editTask.list_id) return
      const idx = this.tasks.findIndex(t => t.id === this.editTask.id)
      if (idx !== -1) {
        this.tasks.splice(idx, 1, {
          ...this.tasks[idx],
          title: this.editTask.title.trim(),
          list_id: this.editTask.list_id,
          cost_sats: Number(this.editTask.cost_sats)
        })
        await this.kvSet(`task_cost:${this.tasks[idx].id}`, String(this.tasks[idx].cost_sats))
        await this.kvSet(`task_list:${this.tasks[idx].id}`, String(this.tasks[idx].list_id))
        await this.saveTasks()
      }
      this.editTaskDialog = false
      this.editTask = null
    },
    async deleteTask(task) {
      const ok = await this.$q.dialog({
        title: 'Delete task',
        message: 'Delete this task?',
        cancel: true,
        persistent: true
      })
      if (!ok) return
      this.tasks = this.tasks.filter(t => t.id !== task.id)
      await this.saveTasks()
      await Promise.all([
        this.kvSet(`task_cost:${task.id}`, ''),
        this.kvSet(`task_list:${task.id}`, ''),
        this.kvSet(`task_paid:${task.id}`, '')
      ])
      await this.loadPaidStatuses()
    },
    async loadAll() {
      if (!this.walletKey) return
      this.loading = true
      try {
        await this.loadListsAndTasks()
        await this.loadPaidStatuses()
        this.connectPaidSockets()
      } catch (err) {
        LNbits.utils.notifyApiError(err)
      } finally {
        this.loading = false
      }
    }
  },
  watch: {},
  created() {
    this.loadAll()
  }
})
