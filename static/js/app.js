// 全局状态
let apis = [];
let currentEditId = null;
let currentLogId = null;
let draggedItem = null;
let expandedIds = new Set();
let editingIds = new Set();
let selectedIds = new Set();

// 分页相关
let currentPage = 1;
let pageSize = 10;

const defaultHeaders = {
    'GET': { 'Accept': 'application/json' },
    'POST': { 'Content-Type': 'application/json', 'Accept': 'application/json' },
    'PUT': { 'Content-Type': 'application/json', 'Accept': 'application/json' },
    'DELETE': { 'Accept': 'application/json' }
};

// 加载API列表
async function loadAPIs() {
    try {
        const res = await fetch('/api/list');
        apis = await res.json() || [];
        renderList();
    } catch (e) {
        showToast('加载失败: ' + e.message, 'error');
    }
}

// 获取过滤后的API列表
function getFilteredAPIs() {
    const search = document.getElementById('searchInput').value.toLowerCase();
    const method = document.getElementById('methodFilter').value;
    
    return apis.filter(api => {
        const matchSearch = !search || 
            (api.name || '').toLowerCase().includes(search) || 
            (api.url || '').toLowerCase().includes(search);
        const matchMethod = !method || api.method === method;
        return matchSearch && matchMethod;
    });
}

// 格式化时间显示 - 显示完整年月日时分秒
function formatTime(timeStr) {
    if (!timeStr) return '-';
    return timeStr;
}

// 渲染列表
function renderList() {
    const filtered = getFilteredAPIs();
    const totalCount = filtered.length;
    const totalPages = Math.ceil(totalCount / pageSize) || 1;
    
    if (currentPage > totalPages) currentPage = totalPages;
    if (currentPage < 1) currentPage = 1;
    
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = Math.min(startIndex + pageSize, totalCount);
    const pageData = filtered.slice(startIndex, endIndex);

    const list = document.getElementById('apiList');
    
    if (totalCount === 0) {
        list.innerHTML = '<div class="empty-state">暂无接口数据，点击"新增接口"开始</div>';
        updateBatchDeleteBtn();
        return;
    }

    const pageIds = pageData.map(api => api.id);
    const allSelected = pageIds.length > 0 && pageIds.every(id => selectedIds.has(id));

    // 表头 - 添加更新时间列
    let html = `
        <div class="api-header">
            <div class="header-cell"><input type="checkbox" class="header-checkbox" id="selectAll" ${allSelected ? 'checked' : ''} onchange="toggleSelectAll(this.checked)"></div>
            <div class="header-cell">序号</div>
            <div class="header-cell"></div>
            <div class="header-cell"></div>
            <div class="header-cell left">名称</div>
            <div class="header-cell">类型</div>
            <div class="header-cell left">URL地址</div>
            <div class="header-cell">请求头</div>
            <div class="header-cell">响应</div>
            <div class="header-cell">更新时间</div>
            <div class="header-cell">操作</div>
        </div>
    `;

    // 数据行
    html += pageData.map((api, idx) => {
        const isExpanded = expandedIds.has(api.id);
        const isEditing = editingIds.has(api.id);
        const isSelected = selectedIds.has(api.id);
        const globalIndex = startIndex + idx + 1;
        return `
        <div class="api-item" data-id="${api.id}">
            <div class="api-row">
                <input type="checkbox" class="row-checkbox" ${isSelected ? 'checked' : ''} onchange="toggleSelect('${api.id}', this.checked)">
                <span class="row-index">${globalIndex}</span>
                <span class="drag-handle" draggable="true" title="拖动排序">⋮⋮</span>
                <button class="expand-btn ${isExpanded ? 'expanded' : ''}" id="expand-${api.id}" onclick="toggleDetail('${api.id}')">▶</button>
                <div class="api-name clickable-area" title="${api.name || ''}" onclick="toggleDetail('${api.id}')">${api.name || '未命名'}</div>
                <span class="method-badge method-${api.method} clickable-area" onclick="toggleDetail('${api.id}')">${api.method}</span>
                <div class="api-url clickable-area" title="${api.url || ''}" onclick="toggleDetail('${api.id}')">${api.url || '/'}</div>
                <div class="header-preview" onclick="openHeaders('${api.id}')" title="点击编辑">${Object.keys(api.headers || {}).length} 个头</div>
                <div class="response-preview" onclick="openResponse('${api.id}')" title="点击编辑">${(api.responseBody || '').length} 字符</div>
                <div class="update-time" title="${api.updatedAt || ''}">${formatTime(api.updatedAt)}</div>
                <div class="actions">
                    <button class="action-btn ${isEditing ? 'btn-save' : 'btn-edit'}" onclick="toggleEdit('${api.id}')">${isEditing ? '保存' : '编辑'}</button>
                    <button class="action-btn btn-log" onclick="openLogs('${api.id}')">日志</button>
                    <button class="action-btn btn-delete" onclick="deleteAPI('${api.id}')">删除</button>
                </div>
            </div>
            <div class="api-detail ${isExpanded ? 'show' : ''}" id="detail-${api.id}">${renderDetail(api, isEditing)}</div>
        </div>
    `}).join('');

    // 分页
    html += renderPagination(totalCount, totalPages);

    list.innerHTML = html;
    initDragAndDrop();
    updateBatchDeleteBtn();
}

// 渲染详情面板
function renderDetail(api, isEditing = false) {
    const disabled = isEditing ? '' : 'disabled';
    return `
        <div class="detail-grid">
            <div class="detail-group">
                <label class="detail-label">服务名称</label>
                <input type="text" class="detail-input editable" id="name-${api.id}" value="${api.name || ''}" ${disabled} placeholder="输入服务名称">
            </div>
            <div class="detail-group">
                <label class="detail-label">请求方法</label>
                <select class="detail-input editable" id="method-${api.id}" ${disabled}>
                    <option value="GET" ${api.method === 'GET' ? 'selected' : ''}>GET</option>
                    <option value="POST" ${api.method === 'POST' ? 'selected' : ''}>POST</option>
                    <option value="PUT" ${api.method === 'PUT' ? 'selected' : ''}>PUT</option>
                    <option value="DELETE" ${api.method === 'DELETE' ? 'selected' : ''}>DELETE</option>
                </select>
            </div>
            <div class="detail-group full">
                <label class="detail-label">请求URL</label>
                <input type="text" class="detail-input editable" id="url-${api.id}" value="${api.url || ''}" ${disabled} placeholder="/api/example">
            </div>
            <div class="detail-group full">
                <label class="detail-label">请求头 (JSON格式)</label>
                <textarea class="detail-textarea editable" id="headers-${api.id}" ${disabled} placeholder='{"Content-Type": "application/json"}'>${JSON.stringify(api.headers || {}, null, 2)}</textarea>
            </div>
            <div class="detail-group full">
                <label class="detail-label">响应体 (JSON格式)</label>
                <textarea class="detail-textarea editable" id="response-${api.id}" ${disabled} placeholder='{"code": 200, "data": {}}'>${api.responseBody || ''}</textarea>
            </div>
        </div>
    `;
}

// 渲染分页
function renderPagination(totalCount, totalPages) {
    return `
        <div class="pagination">
            <div class="pagination-info">
                共 <strong>${totalCount}</strong> 条记录，第 <strong>${currentPage}</strong>/${totalPages} 页
            </div>
            <div class="pagination-controls">
                <span style="margin-right: 8px;">每页</span>
                <select class="page-size-select" onchange="changePageSize(this.value)">
                    <option value="10" ${pageSize === 10 ? 'selected' : ''}>10</option>
                    <option value="20" ${pageSize === 20 ? 'selected' : ''}>20</option>
                    <option value="50" ${pageSize === 50 ? 'selected' : ''}>50</option>
                    <option value="100" ${pageSize === 100 ? 'selected' : ''}>100</option>
                </select>
                <span style="margin: 0 8px;">条</span>
                <button class="pagination-btn" onclick="goToPage(1)" ${currentPage === 1 ? 'disabled' : ''}>首页</button>
                <button class="pagination-btn" onclick="goToPage(${currentPage - 1})" ${currentPage === 1 ? 'disabled' : ''}>上一页</button>
                <span class="pagination-current">${currentPage}</span>
                <button class="pagination-btn" onclick="goToPage(${currentPage + 1})" ${currentPage === totalPages ? 'disabled' : ''}>下一页</button>
                <button class="pagination-btn" onclick="goToPage(${totalPages})" ${currentPage === totalPages ? 'disabled' : ''}>末页</button>
            </div>
        </div>
    `;
}

// 分页函数
function goToPage(page) {
    const filtered = getFilteredAPIs();
    const totalPages = Math.ceil(filtered.length / pageSize) || 1;
    if (page >= 1 && page <= totalPages) {
        currentPage = page;
        renderList();
    }
}

function changePageSize(size) {
    pageSize = parseInt(size);
    currentPage = 1;
    renderList();
}

// 选择相关函数
function toggleSelect(id, checked) {
    if (checked) {
        selectedIds.add(id);
    } else {
        selectedIds.delete(id);
    }
    updateSelectAllCheckbox();
    updateBatchDeleteBtn();
}

function toggleSelectAll(checked) {
    const filtered = getFilteredAPIs();
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = Math.min(startIndex + pageSize, filtered.length);
    const pageData = filtered.slice(startIndex, endIndex);
    
    pageData.forEach(api => {
        if (checked) {
            selectedIds.add(api.id);
        } else {
            selectedIds.delete(api.id);
        }
    });
    renderList();
}

function updateSelectAllCheckbox() {
    const filtered = getFilteredAPIs();
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = Math.min(startIndex + pageSize, filtered.length);
    const pageData = filtered.slice(startIndex, endIndex);
    
    const pageIds = pageData.map(api => api.id);
    const allSelected = pageIds.length > 0 && pageIds.every(id => selectedIds.has(id));
    
    const selectAllCheckbox = document.getElementById('selectAll');
    if (selectAllCheckbox) {
        selectAllCheckbox.checked = allSelected;
    }
}

function updateBatchDeleteBtn() {
    const btn = document.getElementById('batchDeleteBtn');
    if (btn) {
        btn.disabled = selectedIds.size === 0;
        btn.innerHTML = selectedIds.size > 0 
            ? `<span>🗑️</span> 批量删除 (${selectedIds.size})`
            : `<span>🗑️</span> 批量删除`;
    }
}

async function batchDelete() {
    if (selectedIds.size === 0) return;
    
    if (!confirm(`确定要删除选中的 ${selectedIds.size} 个接口吗？`)) return;
    
    try {
        for (const id of selectedIds) {
            await fetch('/api/delete', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ id })
            });
            expandedIds.delete(id);
            editingIds.delete(id);
        }
        selectedIds.clear();
        showToast('批量删除成功', 'success');
        loadAPIs();
    } catch (e) {
        showToast('删除失败: ' + e.message, 'error');
    }
}

// 编辑相关函数
function enableEdit(id) {
    editingIds.add(id);
    const inputs = [
        document.getElementById(`name-${id}`),
        document.getElementById(`method-${id}`),
        document.getElementById(`url-${id}`),
        document.getElementById(`headers-${id}`),
        document.getElementById(`response-${id}`)
    ];
    inputs.forEach(input => { if(input) input.disabled = false; });
    
    const btn = document.querySelector(`.api-item[data-id="${id}"] .btn-edit, .api-item[data-id="${id}"] .btn-save`);
    if (btn) {
        btn.textContent = '保存';
        btn.classList.remove('btn-edit');
        btn.classList.add('btn-save');
    }
}

function toggleDetail(id) {
    if (expandedIds.has(id)) {
        expandedIds.delete(id);
    } else {
        expandedIds.add(id);
    }
    const detail = document.getElementById(`detail-${id}`);
    const btn = document.getElementById(`expand-${id}`);
    if (detail) detail.classList.toggle('show');
    if (btn) btn.classList.toggle('expanded');
}

function toggleEdit(id) {
    if (editingIds.has(id)) {
        saveAPI(id);
    } else {
        enableEdit(id);
        if (!expandedIds.has(id)) {
            toggleDetail(id);
        }
    }
}

async function saveAPI(id) {
    const api = apis.find(a => a.id === id) || { id };
    
    api.name = document.getElementById(`name-${id}`)?.value || '';
    api.method = document.getElementById(`method-${id}`)?.value || 'GET';
    api.url = document.getElementById(`url-${id}`)?.value || '';
    
    try {
        api.headers = JSON.parse(document.getElementById(`headers-${id}`)?.value || '{}');
    } catch (e) {
        showToast('请求头JSON格式错误', 'error');
        return;
    }
    
    api.responseBody = document.getElementById(`response-${id}`)?.value || '';

    if (!api.name || !api.url) {
        showToast('服务名称和URL不能为空', 'error');
        return;
    }

    try {
        const res = await fetch('/api/save', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(api)
        });
        const data = await res.json();
        if (data.success) {
            showToast('保存成功', 'success');
            editingIds.delete(id);
            
            // 更新本地数据
            const idx = apis.findIndex(a => a.id === id);
            if (idx !== -1) {
                apis[idx] = data.api || { ...apis[idx], ...api };
            } else if (data.api) {
                // 新增的情况，更新ID
                const newIdx = apis.findIndex(a => a.id === id);
                if (newIdx !== -1) {
                    apis[newIdx] = data.api;
                }
            }
            
            renderList();
        }
    } catch (e) {
        showToast('保存失败: ' + e.message, 'error');
    }
}

async function deleteAPI(id) {
    if (!confirm('确定要删除这个接口吗？')) return;
    
    try {
        await fetch('/api/delete', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ id })
        });
        showToast('删除成功', 'success');
        expandedIds.delete(id);
        editingIds.delete(id);
        selectedIds.delete(id);
        loadAPIs();
    } catch (e) {
        showToast('删除失败: ' + e.message, 'error');
    }
}

function addNewAPI() {
    const newId = 'new-' + Date.now();
    const newApi = {
        id: newId,
        name: '',
        method: 'GET',
        url: '',
        headers: defaultHeaders['GET'],
        responseBody: '{"code": 200, "data": {}, "message": "success"}',
        logs: [],
        updatedAt: new Date().toISOString().replace('T', ' ').substring(0, 19)
    };
    apis.unshift(newApi);
    expandedIds.add(newId);
    editingIds.add(newId);
    currentPage = 1;
    renderList();
    
    setTimeout(() => {
        const nameInput = document.getElementById(`name-${newId}`);
        if (nameInput) nameInput.focus();
    }, 100);
}

// 拖拽排序
function initDragAndDrop() {
    const handles = document.querySelectorAll('.drag-handle');
    const items = document.querySelectorAll('.api-item');
    
    handles.forEach(handle => {
        handle.addEventListener('dragstart', handleDragStart);
        handle.addEventListener('dragend', handleDragEnd);
    });
    
    items.forEach(item => {
        item.addEventListener('dragover', handleDragOver);
        item.addEventListener('dragenter', handleDragEnter);
        item.addEventListener('dragleave', handleDragLeave);
        item.addEventListener('drop', handleDrop);
    });
}

function handleDragStart(e) {
    const item = this.closest('.api-item');
    draggedItem = item;
    item.classList.add('dragging');
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', item.dataset.id);
}

function handleDragEnd(e) {
    const item = this.closest('.api-item');
    item.classList.remove('dragging');
    document.querySelectorAll('.api-item').forEach(item => {
        item.classList.remove('drag-over');
    });
    draggedItem = null;
}

function handleDragOver(e) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
}

function handleDragEnter(e) {
    e.preventDefault();
    if (this !== draggedItem) {
        this.classList.add('drag-over');
    }
}

function handleDragLeave(e) {
    this.classList.remove('drag-over');
}

async function handleDrop(e) {
    e.preventDefault();
    this.classList.remove('drag-over');
    
    if (draggedItem && this !== draggedItem) {
        const draggedId = draggedItem.dataset.id;
        const targetId = this.dataset.id;
        
        const draggedIndex = apis.findIndex(a => a.id === draggedId);
        const targetIndex = apis.findIndex(a => a.id === targetId);
        
        if (draggedIndex !== -1 && targetIndex !== -1) {
            const [removed] = apis.splice(draggedIndex, 1);
            apis.splice(targetIndex, 0, removed);
            
            await saveOrder();
            renderList();
        }
    }
}

async function saveOrder() {
    try {
        await fetch('/api/reorder', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ids: apis.map(a => a.id) })
        });
    } catch (e) {
        console.error('保存排序失败', e);
    }
}

// 弹窗相关函数
function openResponse(id) {
    currentEditId = id;
    const api = apis.find(a => a.id === id);
    document.getElementById('responseEditor').value = api?.responseBody || '';
    document.getElementById('responseModal').classList.add('show');
}

async function saveResponse() {
    const content = document.getElementById('responseEditor').value;
    const api = apis.find(a => a.id === currentEditId);
    if (!api) return;
    api.responseBody = content;
    
    const textarea = document.getElementById(`response-${currentEditId}`);
    if (textarea) textarea.value = content;
    
    await saveAPIData(api);
    closeModal('responseModal');
}

function openHeaders(id) {
    currentEditId = id;
    const api = apis.find(a => a.id === id);
    document.getElementById('headersEditor').value = JSON.stringify(api?.headers || {}, null, 2);
    document.getElementById('headersModal').classList.add('show');
}

async function saveHeaders() {
    try {
        const content = document.getElementById('headersEditor').value;
        const headers = JSON.parse(content);
        const api = apis.find(a => a.id === currentEditId);
        if (!api) return;
        api.headers = headers;
        
        const textarea = document.getElementById(`headers-${currentEditId}`);
        if (textarea) textarea.value = JSON.stringify(headers, null, 2);
        
        await saveAPIData(api);
        closeModal('headersModal');
    } catch (e) {
        showToast('JSON格式错误', 'error');
    }
}

async function saveAPIData(api) {
    if (!api.name || !api.url) {
        showToast('请先完善服务名称和URL', 'error');
        return;
    }
    
    try {
        const res = await fetch('/api/save', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(api)
        });
        const data = await res.json();
        if (data.success) {
            showToast('保存成功', 'success');
            if (data.api) {
                const idx = apis.findIndex(a => a.id === api.id);
                if (idx !== -1) {
                    apis[idx] = data.api;
                }
            }
            renderList();
        }
    } catch (e) {
        showToast('保存失败: ' + e.message, 'error');
    }
}

async function openLogs(id) {
    currentLogId = id;
    
    try {
        const res = await fetch(`/api/logs?id=${id}`);
        const logs = await res.json() || [];
        
        const logList = document.getElementById('logList');
        if (logs.length === 0) {
            logList.innerHTML = '<div class="empty-state">暂无请求日志</div>';
        } else {
            logList.innerHTML = logs.reverse().map(log => `
                <div class="log-item">
                    <div class="log-header">
                        <span>
                            <span class="status-dot ${log.statusCode === 200 ? 'status-success' : 'status-error'}"></span>
                            <strong>${log.method}</strong> ${log.url}
                        </span>
                        <span>${log.timestamp}</span>
                    </div>
                    ${log.requestBody ? `<div class="log-body">${log.requestBody}</div>` : ''}
                    ${log.error ? `<div class="log-body" style="color:#d32f2f">${log.error}</div>` : ''}
                </div>
            `).join('');
        }
        
        document.getElementById('logsModal').classList.add('show');
    } catch (e) {
        showToast('加载日志失败', 'error');
    }
}

async function clearLogs() {
    if (!confirm('确定要清空日志吗？')) return;
    
    try {
        await fetch('/api/clear-logs', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ id: currentLogId })
        });
        showToast('日志已清空', 'success');
        document.getElementById('logList').innerHTML = '<div class="empty-state">暂无请求日志</div>';
    } catch (e) {
        showToast('清空失败', 'error');
    }
}

function closeModal(id) {
    document.getElementById(id).classList.remove('show');
}

function formatJSON() {
    try {
        const editor = document.getElementById('responseEditor');
        const json = JSON.parse(editor.value);
        editor.value = JSON.stringify(json, null, 2);
    } catch (e) {
        showToast('JSON格式错误，无法格式化', 'error');
    }
}

function formatHeadersJSON() {
    try {
        const editor = document.getElementById('headersEditor');
        const json = JSON.parse(editor.value);
        editor.value = JSON.stringify(json, null, 2);
    } catch (e) {
        showToast('JSON格式错误，无法格式化', 'error');
    }
}

function showToast(msg, type = '') {
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = msg;
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

// 初始化事件监听
document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('searchInput').addEventListener('input', () => { currentPage = 1; renderList(); });
    document.getElementById('methodFilter').addEventListener('change', () => { currentPage = 1; renderList(); });

    document.querySelectorAll('.modal').forEach(modal => {
        modal.addEventListener('click', (e) => {
            if (e.target === modal) modal.classList.remove('show');
        });
    });

    loadAPIs();
});
