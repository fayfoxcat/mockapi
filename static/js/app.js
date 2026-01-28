// 全局状态
let apis = [];
let currentEditId = null;
let currentLogId = null;
let draggedItem = null;
let expandedIds = new Set();
let editingIds = new Set();
let selectedIds = new Set();
let currentResponseType = 'json';
let uploadedFileInfo = null;

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
                <div class="response-preview" onclick="openResponse('${api.id}')" title="点击编辑">${getResponsePreview(api)}</div>
                <div class="update-time" title="${api.updatedAt || ''}">${formatTime(api.updatedAt)}</div>
                <div class="actions">
                    <button class="action-btn ${isEditing ? 'btn-save' : 'btn-edit'}" onclick="toggleEdit('${api.id}')">${isEditing ? '保存' : '编辑'}</button>
                    <button class="action-btn btn-copy" onclick="copyCurl('${api.id}')" title="复制CURL命令">CURL</button>
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
    const responseTypeDisplay = api.responseType === 'file' ? 
        `<span class="response-type-badge response-type-file">文件</span>` : 
        `<span class="response-type-badge response-type-json">JSON</span>`;
    
    let responseContent = '';
    if (api.responseType === 'file' && api.fileName) {
        responseContent = `文件: ${api.fileName}`;
    } else {
        responseContent = api.responseBody || '';
    }
    
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
                <label class="detail-label">响应类型</label>
                <select class="detail-input editable" id="responseType-${api.id}" ${disabled} onchange="toggleDetailResponseType('${api.id}', this.value)">
                    <option value="json" ${api.responseType === 'json' || !api.responseType ? 'selected' : ''}>JSON响应</option>
                    <option value="file" ${api.responseType === 'file' ? 'selected' : ''}>文件响应</option>
                </select>
            </div>
            <div class="detail-group full" id="jsonSection-${api.id}" style="${api.responseType === 'file' ? 'display: none;' : ''}">
                <label class="detail-label">响应体 (JSON格式)</label>
                <textarea class="detail-textarea editable" id="response-${api.id}" ${disabled} placeholder='{"code": 200, "data": {}}'>${api.responseType === 'file' ? '' : responseContent}</textarea>
            </div>
            <div class="detail-group full" id="fileSection-${api.id}" style="${api.responseType === 'file' ? '' : 'display: none;'}">
                <label class="detail-label">文件信息</label>
                <div class="file-preview" id="filePreview-${api.id}">
                    ${api.responseType === 'file' && api.fileName ? 
                        `<div style="color: #4CAF50; font-size: 12px; margin-bottom: 4px;">📁 当前文件:</div>
                         <div style="font-weight: 500;">${api.fileName}</div>
                         <div style="color: #666; font-size: 11px;">${api.contentType || 'unknown'}</div>` : 
                        '未选择文件'}
                </div>
                ${isEditing ? `
                <div class="file-upload-inline">
                    <input type="file" id="fileInput-${api.id}" accept="*/*" onchange="handleDetailFileSelect('${api.id}', this)">
                    <button type="button" class="btn btn-secondary btn-small" onclick="uploadDetailFile('${api.id}')" id="uploadBtn-${api.id}">
                        ${api.fileName ? '重新上传' : '上传文件'}
                    </button>
                </div>
                ` : ''}
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
        document.getElementById(`response-${id}`),
        document.getElementById(`responseType-${id}`)
    ];
    inputs.forEach(input => { if(input) input.disabled = false; });
    
    // 如果当前是文件响应类型，需要重新渲染详情面板以显示文件上传控件
    const api = apis.find(a => a.id === id);
    if (api && api.responseType === 'file') {
        const detailElement = document.getElementById(`detail-${id}`);
        if (detailElement) {
            detailElement.innerHTML = renderDetail(api, true);
        }
    }
    
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
    
    // 获取响应类型
    const responseTypeSelect = document.getElementById(`responseType-${id}`);
    const selectedType = responseTypeSelect?.value || 'json';
    
    if (selectedType === 'json') {
        api.responseType = 'json';
        api.responseBody = document.getElementById(`response-${id}`)?.value || '';
        // 如果从文件类型切换到JSON类型，清除文件相关字段
        if (api.responseType !== 'json') {
            api.fileName = null;
            api.filePath = null;
            api.contentType = null;
        }
    } else {
        // 文件响应类型
        api.responseType = 'file';
        api.responseBody = '';
        
        // 检查是否有文件信息（新增接口或已有文件）
        if (!api.fileName && !api.filePath) {
            showToast('文件响应类型需要上传文件', 'error');
            return;
        }
    }

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
        responseType: 'json',
        fileName: null,
        filePath: null,
        contentType: null,
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
            logList.innerHTML = logs.reverse().map(log => {
                const statusClass = log.statusCode === 200 ? 'log-success' : 'log-error';
                const statusDot = log.statusCode === 200 ? 'status-success' : 'status-error';
                
                // 格式化请求头显示
                const headersDisplay = Object.keys(log.headers || {}).length > 0 ? 
                    Object.entries(log.headers).map(([key, value]) => 
                        `<div class="header-item"><span class="header-key">${key}:</span> <span class="header-value">${value}</span></div>`
                    ).join('') : '<div class="no-headers">无请求头</div>';
                
                return `
                <div class="log-item ${statusClass}">
                    <div class="log-header">
                        <div class="log-status-method">
                            <span class="status-dot ${statusDot}"></span>
                            <strong class="log-method">${log.method}</strong>
                            <span class="method-badge method-${log.method}">${log.statusCode}</span>
                        </div>
                        <span class="log-timestamp">${log.timestamp}</span>
                    </div>
                    <div class="log-url-section">
                        <div class="log-url-label">🔗 请求地址:</div>
                        <div class="log-url-value">${log.url}</div>
                    </div>
                    <div class="log-meta">
                        <div class="log-meta-item">
                            <span class="log-meta-label">🌐 客户端IP</span>
                            <span class="log-meta-value">${log.clientIp || 'Unknown'}</span>
                        </div>
                        <div class="log-meta-item">
                            <span class="log-meta-label">🖥️ 用户代理</span>
                            <span class="log-meta-value" title="${log.userAgent || 'Unknown'}">${formatUserAgent(log.userAgent)}</span>
                        </div>
                    </div>
                    <div class="log-headers-section">
                        <div class="log-headers-label">📋 请求头:</div>
                        <div class="log-headers-content">
                            ${headersDisplay}
                        </div>
                    </div>
                    ${log.requestBody ? `
                    <div class="log-request-body">
                        <div class="log-meta-label">📤 请求体:</div>
                        <div class="log-body">${log.requestBody}</div>
                    </div>
                    ` : ''}
                    ${log.error ? `
                    <div class="log-error-section">
                        <div class="log-meta-label log-error-label">❌ 错误信息:</div>
                        <div class="log-body log-error">${log.error}</div>
                    </div>
                    ` : ''}
                </div>
            `}).join('');
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

// 复制CURL命令
function copyCurl(id) {
    const api = apis.find(a => a.id === id);
    if (!api) return;
    
    let curlCmd = `curl -X ${api.method}`;
    
    // 添加URL
    curlCmd += ` "http://localhost:${window.location.port || '8344'}${api.url}"`;
    
    // 添加请求头
    if (api.headers && Object.keys(api.headers).length > 0) {
        for (const [key, value] of Object.entries(api.headers)) {
            curlCmd += ` \\\n  -H "${key}: ${value}"`;
        }
    }
    
    // 如果是POST/PUT/DELETE请求，添加示例请求体
    if (['POST', 'PUT', 'DELETE'].includes(api.method)) {
        curlCmd += ` \\\n  -d '{"key": "value"}'`;
    }
    
    // 兼容性检查和复制到剪贴板
    if (navigator.clipboard && navigator.clipboard.writeText) {
        // 现代浏览器支持 Clipboard API
        navigator.clipboard.writeText(curlCmd).then(() => {
            showToast('CURL命令已复制到剪贴板', 'success');
        }).catch(err => {
            console.error('Clipboard API failed:', err);
            fallbackCopyTextToClipboard(curlCmd);
        });
    } else {
        // 降级方案：使用传统方法
        fallbackCopyTextToClipboard(curlCmd);
    }
}

// 降级复制方案
function fallbackCopyTextToClipboard(text) {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    
    // 避免在页面上显示
    textArea.style.position = 'fixed';
    textArea.style.top = '0';
    textArea.style.left = '0';
    textArea.style.width = '2em';
    textArea.style.height = '2em';
    textArea.style.padding = '0';
    textArea.style.border = 'none';
    textArea.style.outline = 'none';
    textArea.style.boxShadow = 'none';
    textArea.style.background = 'transparent';
    
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    
    try {
        const successful = document.execCommand('copy');
        if (successful) {
            showToast('CURL命令已复制到剪贴板', 'success');
        } else {
            showCurlModal(text);
        }
    } catch (err) {
        console.error('Fallback copy failed:', err);
        showCurlModal(text);
    }
    
    document.body.removeChild(textArea);
}

// 显示CURL命令弹窗（最后的降级方案）
function showCurlModal(curlCmd) {
    // 创建弹窗
    const modal = document.createElement('div');
    modal.className = 'modal show';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>📋 CURL命令</h3>
                <button class="modal-close" onclick="this.closest('.modal').remove()">✕</button>
            </div>
            <div class="modal-body">
                <p style="margin-bottom: 10px; color: #666;">请手动复制以下命令：</p>
                <textarea class="modal-textarea" readonly style="min-height: 150px;">${curlCmd}</textarea>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal').remove()">关闭</button>
                <button class="btn btn-primary" onclick="selectAllText(this)">全选</button>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    
    // 自动选中文本
    const textarea = modal.querySelector('textarea');
    textarea.focus();
    textarea.select();
    
    showToast('复制功能不可用，请手动复制', 'error');
}

// 全选文本辅助函数
function selectAllText(button) {
    const textarea = button.closest('.modal').querySelector('textarea');
    textarea.focus();
    textarea.select();
    showToast('文本已全选，请按 Ctrl+C 复制', 'info');
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
// 获取响应预览文本
function getResponsePreview(api) {
    if (api.responseType === 'file') {
        return api.fileName ? `📁 ${api.fileName}` : '📁 文件响应';
    } else {
        return `${(api.responseBody || '').length} 字符`;
    }
}

// 切换详情面板中的响应类型
function toggleDetailResponseType(id, type) {
    const jsonSection = document.getElementById(`jsonSection-${id}`);
    const fileSection = document.getElementById(`fileSection-${id}`);
    
    if (type === 'file') {
        if (jsonSection) jsonSection.style.display = 'none';
        if (fileSection) fileSection.style.display = 'block';
    } else {
        if (jsonSection) jsonSection.style.display = 'block';
        if (fileSection) fileSection.style.display = 'none';
    }
    
    // 如果是编辑模式，需要重新渲染以显示正确的控件
    if (editingIds.has(id)) {
        const api = apis.find(a => a.id === id);
        if (api) {
            // 临时更新响应类型
            api.responseType = type;
            // 重新渲染详情面板
            const detailElement = document.getElementById(`detail-${id}`);
            if (detailElement) {
                detailElement.innerHTML = renderDetail(api, true);
            }
        }
    }
}

// 切换响应类型（弹窗中）
function toggleResponseType(type) {
    currentResponseType = type;
    const jsonSection = document.getElementById('jsonResponseSection');
    const fileSection = document.getElementById('fileResponseSection');
    
    if (type === 'file') {
        jsonSection.style.display = 'none';
        fileSection.style.display = 'block';
    } else {
        jsonSection.style.display = 'block';
        fileSection.style.display = 'none';
    }
}

// 处理文件选择
function handleFileSelect(input) {
    const file = input.files[0];
    const fileInfo = document.getElementById('fileInfo');
    const uploadBtn = document.getElementById('uploadBtn');
    
    if (file) {
        document.getElementById('fileName').textContent = file.name;
        document.getElementById('fileSize').textContent = formatFileSize(file.size);
        document.getElementById('fileType').textContent = file.type || 'unknown';
        
        fileInfo.style.display = 'block';
        uploadBtn.disabled = false;
        uploadBtn.textContent = '上传文件';
    } else {
        fileInfo.style.display = 'none';
        uploadBtn.disabled = true;
        uploadBtn.textContent = '上传文件';
    }
}

// 格式化文件大小
function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 上传文件
async function uploadFile() {
    const fileInput = document.getElementById('fileInput');
    const file = fileInput.files[0];
    
    if (!file) {
        showToast('请选择文件', 'error');
        return;
    }
    
    const formData = new FormData();
    formData.append('file', file);
    
    try {
        const uploadBtn = document.getElementById('uploadBtn');
        uploadBtn.disabled = true;
        uploadBtn.textContent = '上传中...';
        
        const res = await fetch('/api/upload', {
            method: 'POST',
            body: formData
        });
        
        const data = await res.json();
        
        if (data.success) {
            uploadedFileInfo = {
                fileName: data.fileName,
                filePath: data.filePath,
                contentType: data.contentType
            };
            
            showToast('文件上传成功', 'success');
            uploadBtn.textContent = '重新上传';
        } else {
            showToast('文件上传失败', 'error');
        }
    } catch (e) {
        showToast('上传失败: ' + e.message, 'error');
    } finally {
        const uploadBtn = document.getElementById('uploadBtn');
        uploadBtn.disabled = false;
        if (uploadBtn.textContent === '上传中...') {
            uploadBtn.textContent = '上传文件';
        }
    }
}

// 更新openResponse函数
function openResponse(id) {
    currentEditId = id;
    const api = apis.find(a => a.id === id);
    
    // 重置状态
    uploadedFileInfo = null;
    currentResponseType = api?.responseType || 'json';
    
    // 设置响应类型单选按钮
    const jsonRadio = document.querySelector('input[name="responseType"][value="json"]');
    const fileRadio = document.querySelector('input[name="responseType"][value="file"]');
    
    if (currentResponseType === 'file') {
        fileRadio.checked = true;
        jsonRadio.checked = false;
    } else {
        jsonRadio.checked = true;
        fileRadio.checked = false;
    }
    
    // 切换显示区域
    toggleResponseType(currentResponseType);
    
    // 设置内容
    if (currentResponseType === 'file') {
        // 显示文件信息
        if (api?.fileName) {
            const fileInfo = document.getElementById('fileInfo');
            document.getElementById('fileName').textContent = api.fileName;
            document.getElementById('fileSize').textContent = '已上传';
            document.getElementById('fileType').textContent = api.contentType || 'unknown';
            fileInfo.style.display = 'block';
            
            // 保存现有文件信息
            uploadedFileInfo = {
                fileName: api.fileName,
                filePath: api.filePath,
                contentType: api.contentType
            };
        }
    } else {
        document.getElementById('responseEditor').value = api?.responseBody || '';
    }
    
    document.getElementById('responseModal').classList.add('show');
}

// 更新saveResponse函数
async function saveResponse() {
    const api = apis.find(a => a.id === currentEditId);
    if (!api) return;
    
    if (currentResponseType === 'json') {
        const content = document.getElementById('responseEditor').value;
        api.responseBody = content;
        api.responseType = 'json';
        // 清除文件相关字段
        api.fileName = null;
        api.filePath = null;
        api.contentType = null;
        
        const textarea = document.getElementById(`response-${currentEditId}`);
        if (textarea) textarea.value = content;
    } else {
        // 文件响应
        if (!uploadedFileInfo) {
            showToast('请先上传文件', 'error');
            return;
        }
        
        api.responseType = 'file';
        api.fileName = uploadedFileInfo.fileName;
        api.filePath = uploadedFileInfo.filePath;
        api.contentType = uploadedFileInfo.contentType;
        api.responseBody = '';
        
        // 更新文件预览
        const filePreview = document.getElementById(`filePreview-${currentEditId}`);
        if (filePreview) {
            filePreview.textContent = `文件: ${uploadedFileInfo.fileName} (${uploadedFileInfo.contentType})`;
        }
    }
    
    await saveAPIData(api);
    closeModal('responseModal');
}
// 处理详情面板中的文件选择
function handleDetailFileSelect(id, input) {
    const file = input.files[0];
    const uploadBtn = document.getElementById(`uploadBtn-${id}`);
    const api = apis.find(a => a.id === id);
    
    if (file) {
        // 选择了新文件
        uploadBtn.disabled = false;
        uploadBtn.textContent = '上传文件';
        
        // 显示文件信息预览
        const filePreview = document.getElementById(`filePreview-${id}`);
        if (filePreview) {
            filePreview.innerHTML = `
                <div style="color: #666; font-size: 12px; margin-bottom: 4px;">准备上传:</div>
                <div style="font-weight: 500;">${file.name}</div>
                <div style="color: #666; font-size: 11px;">${formatFileSize(file.size)} - ${file.type || 'unknown'}</div>
            `;
        }
    } else {
        // 没有选择文件，但如果已有文件，按钮仍然可用于重新上传
        if (api && api.fileName) {
            uploadBtn.disabled = false;
            uploadBtn.textContent = '重新上传';
        } else {
            uploadBtn.disabled = true;
            uploadBtn.textContent = '上传文件';
        }
        
        // 恢复原始文件信息
        const filePreview = document.getElementById(`filePreview-${id}`);
        if (filePreview && api) {
            if (api.responseType === 'file' && api.fileName) {
                filePreview.innerHTML = `
                    <div style="color: #4CAF50; font-size: 12px; margin-bottom: 4px;">📁 当前文件:</div>
                    <div style="font-weight: 500;">${api.fileName}</div>
                    <div style="color: #666; font-size: 11px;">${api.contentType || 'unknown'}</div>
                `;
            } else {
                filePreview.textContent = '未选择文件';
            }
        }
    }
}

// 上传详情面板中的文件
async function uploadDetailFile(id) {
    const fileInput = document.getElementById(`fileInput-${id}`);
    const file = fileInput.files[0];
    const api = apis.find(a => a.id === id);
    
    // 如果没有选择新文件，但有已有文件，提示用户选择新文件
    if (!file) {
        if (api && api.fileName) {
            showToast('请选择要上传的新文件', 'info');
            fileInput.click(); // 自动打开文件选择对话框
        } else {
            showToast('请选择文件', 'error');
        }
        return;
    }
    
    const formData = new FormData();
    formData.append('file', file);
    
    try {
        const uploadBtn = document.getElementById(`uploadBtn-${id}`);
        const originalText = uploadBtn.textContent;
        uploadBtn.disabled = true;
        uploadBtn.textContent = '上传中...';
        
        const res = await fetch('/api/upload', {
            method: 'POST',
            body: formData
        });
        
        const data = await res.json();
        
        if (data.success) {
            // 更新API对象
            if (api) {
                api.fileName = data.fileName;
                api.filePath = data.filePath;
                api.contentType = data.contentType;
                
                // 更新文件预览
                const filePreview = document.getElementById(`filePreview-${id}`);
                if (filePreview) {
                    filePreview.innerHTML = `
                        <div style="color: #4CAF50; font-size: 12px; margin-bottom: 4px;">✅ 上传成功:</div>
                        <div style="font-weight: 500;">${data.fileName}</div>
                        <div style="color: #666; font-size: 11px;">${data.contentType}</div>
                    `;
                }
                
                // 清空文件输入框
                fileInput.value = '';
            }
            
            showToast('文件上传成功', 'success');
            uploadBtn.textContent = '重新上传';
        } else {
            showToast('文件上传失败', 'error');
            uploadBtn.textContent = originalText;
        }
    } catch (e) {
        showToast('上传失败: ' + e.message, 'error');
        const uploadBtn = document.getElementById(`uploadBtn-${id}`);
        uploadBtn.textContent = '上传文件';
    } finally {
        const uploadBtn = document.getElementById(`uploadBtn-${id}`);
        uploadBtn.disabled = false;
    }
}
// 格式化User-Agent显示
function formatUserAgent(userAgent) {
    if (!userAgent || userAgent === 'Unknown') {
        return 'Unknown';
    }
    
    // 简化常见浏览器的显示
    if (userAgent.includes('Chrome') && !userAgent.includes('Edg')) {
        const match = userAgent.match(/Chrome\/([0-9.]+)/);
        return match ? `Chrome ${match[1]}` : 'Chrome';
    } else if (userAgent.includes('Firefox')) {
        const match = userAgent.match(/Firefox\/([0-9.]+)/);
        return match ? `Firefox ${match[1]}` : 'Firefox';
    } else if (userAgent.includes('Safari') && !userAgent.includes('Chrome')) {
        const match = userAgent.match(/Version\/([0-9.]+).*Safari/);
        return match ? `Safari ${match[1]}` : 'Safari';
    } else if (userAgent.includes('Edg')) {
        const match = userAgent.match(/Edg\/([0-9.]+)/);
        return match ? `Edge ${match[1]}` : 'Edge';
    } else if (userAgent.includes('curl')) {
        const match = userAgent.match(/curl\/([0-9.]+)/);
        return match ? `curl ${match[1]}` : 'curl';
    } else if (userAgent.includes('Postman')) {
        return 'Postman';
    } else if (userAgent.includes('Insomnia')) {
        return 'Insomnia';
    } else {
        // 截取前50个字符避免过长
        return userAgent.length > 50 ? userAgent.substring(0, 50) + '...' : userAgent;
    }
}