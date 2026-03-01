/**
 * 极简 hash 路由
 */
const routes = {}
let _contentEl = null
let _loadId = 0
let _currentCleanup = null
let _initialized = false

export function registerRoute(path, loader) {
  routes[path] = loader
}

export function navigate(path) {
  window.location.hash = path
}

export function initRouter(contentEl) {
  _contentEl = contentEl
  if (!_initialized) {
    window.addEventListener('hashchange', () => loadRoute())
    _initialized = true
  }
  loadRoute()
}

async function loadRoute() {
  const hash = window.location.hash.slice(1) || '/dashboard'
  const loader = routes[hash]
  if (!loader || !_contentEl) return

  // 竞态防护：记录本次加载 ID
  const thisLoad = ++_loadId

  // 清理上一个页面
  if (_currentCleanup) {
    try { _currentCleanup() } catch (_) {}
    _currentCleanup = null
  }

  _contentEl.innerHTML = ''

  // 显示加载动画
  const loader_el = document.createElement('div')
  loader_el.className = 'page-loader'
  loader_el.innerHTML = `
    <div class="page-loader-spinner"></div>
    <div class="page-loader-text">加载中...</div>
  `
  _contentEl.appendChild(loader_el)

  const mod = await loader()

  // 如果加载期间路由又变了，丢弃本次结果
  if (thisLoad !== _loadId) return

  const page = mod.render ? await mod.render() : mod.default ? await mod.default() : mod
  if (thisLoad !== _loadId) return

  // 移除加载动画，插入页面内容
  _contentEl.innerHTML = ''
  if (typeof page === 'string') {
    _contentEl.innerHTML = page
  } else if (page instanceof HTMLElement) {
    _contentEl.appendChild(page)
  }

  // 保存页面清理函数
  _currentCleanup = mod.cleanup || null

  // 更新侧边栏激活状态
  document.querySelectorAll('.nav-item').forEach(item => {
    item.classList.toggle('active', item.dataset.route === hash)
  })
}

export function getCurrentRoute() {
  return window.location.hash.slice(1) || '/dashboard'
}
