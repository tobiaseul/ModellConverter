// UI Elements
const dropZone = document.getElementById('drop-zone');
const dropZoneContent = document.getElementById('drop-zone-content');
const fileInput = document.getElementById('file-input');
const browseBtn = document.getElementById('browse-btn');
const browseZipBtn = document.getElementById('browse-zip-btn');
const browseFolderBtn = document.getElementById('browse-folder-btn');
const fromFormat = document.getElementById('from-format');
const toFormat = document.getElementById('to-format');
const convertBtn = document.getElementById('convert-btn');
const statusMessage = document.getElementById('status-message');

// State — exactly one of these is set at a time
let selectedFile = null;   // File object (single model file, read via FileReader)
let selectedPath = null;   // string path (ZIP or folder, via Tauri dialog)
let isBatch = false;

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    setupDragAndDrop();
    setupBrowseButtons();
    setupFormatSelectors();
    convertBtn.addEventListener('click', handleConvert);
});

// ============================================================================
// Drag and Drop (single model files only)
// ============================================================================

function setupDragAndDrop() {
    dropZone.addEventListener('dragover', (e) => {
        e.preventDefault(); e.stopPropagation();
        dropZone.classList.add('drag-over');
    });
    dropZone.addEventListener('dragleave', (e) => {
        e.preventDefault(); e.stopPropagation();
        dropZone.classList.remove('drag-over');
    });
    dropZone.addEventListener('drop', (e) => {
        e.preventDefault(); e.stopPropagation();
        dropZone.classList.remove('drag-over');
        if (e.dataTransfer.files.length > 0) selectFile(e.dataTransfer.files[0]);
    });
    dropZone.addEventListener('click', () => fileInput.click());
}

// ============================================================================
// Browse Buttons
// ============================================================================

function setupBrowseButtons() {
    browseBtn.addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); fileInput.click(); });
    browseZipBtn.addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); pickZip(); });
    browseFolderBtn.addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); pickFolder(); });

    fileInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) selectFile(e.target.files[0]);
    });
}

async function pickZip() {
    const { open } = window.__TAURI__.dialog;
    const path = await open({ multiple: false, filters: [{ name: 'ZIP Archive', extensions: ['zip'] }] });
    if (path) selectBatchPath(path);
}

async function pickFolder() {
    const { open } = window.__TAURI__.dialog;
    const path = await open({ directory: true, multiple: false });
    if (path) selectBatchPath(path);
}

// ============================================================================
// Selection
// ============================================================================

function selectFile(file) {
    selectedFile = file;
    selectedPath = null;
    isBatch = false;
    updateDropZone(file.name, false);
    convertBtn.textContent = 'Convert & Save';
    convertBtn.disabled = false;
    clearStatus();
}

function selectBatchPath(path) {
    selectedFile = null;
    selectedPath = path;
    isBatch = true;
    const name = path.replace(/\\/g, '/').split('/').filter(Boolean).pop() || path;
    updateDropZone(name, true);
    convertBtn.textContent = 'Convert All';
    convertBtn.disabled = false;
    clearStatus();
}

function updateDropZone(name, batch) {
    dropZoneContent.innerHTML = `
        <svg class="drop-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <p class="drop-text">${name}${batch ? ' <span class="batch-badge">batch</span>' : ''}</p>
        <p class="drop-subtext"><a href="#" onclick="event.preventDefault(); resetSelection();">Change</a></p>
    `;
}

function resetSelection() {
    selectedFile = null;
    selectedPath = null;
    isBatch = false;
    dropZoneContent.innerHTML = `
        <svg class="drop-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2v20M2 12h20"></path>
        </svg>
        <p class="drop-text">Drop model file here</p>
        <p class="drop-subtext">
            or <a href="#" id="browse-btn">browse file</a>
            &nbsp;·&nbsp; <a href="#" id="browse-zip-btn">select ZIP</a>
            &nbsp;·&nbsp; <a href="#" id="browse-folder-btn">select folder</a>
        </p>
    `;
    // Re-bind links after innerHTML reset
    document.getElementById('browse-btn').addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); fileInput.click(); });
    document.getElementById('browse-zip-btn').addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); pickZip(); });
    document.getElementById('browse-folder-btn').addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); pickFolder(); });
    convertBtn.textContent = 'Convert & Save';
    convertBtn.disabled = true;
    clearStatus();
}

// ============================================================================
// Format Selectors
// ============================================================================

function setupFormatSelectors() {
    fromFormat.addEventListener('change', () => {});
    // Target format is fixed to EdgeTX
}

// ============================================================================
// Conversion
// ============================================================================

async function handleConvert() {
    convertBtn.classList.add('loading');
    showInfo(isBatch ? 'Converting...' : 'Converting...');

    try {
        if (isBatch) {
            await runBatchConvert();
        } else {
            await runSingleConvert();
        }
    } catch (error) {
        showError(`Error: ${error}`);
    } finally {
        convertBtn.classList.remove('loading');
    }
}

async function runSingleConvert() {
    if (!selectedFile) { showError('No file selected'); return; }

    const fileBytes = await readFileAsBytes(selectedFile);
    const targetExt = getExtensionForFormat(toFormat.value);
    const outputName = getOutputFileName(selectedFile.name, targetExt);

    const { invoke } = window.__TAURI__.core;
    const formatMap = { 'edgetx': 'Edgetx', 'ethos': 'Ethos', 'jeti': 'JetiDuplex' };
    const convertedBytes = await invoke('convert_model', {
        input_bytes: Array.from(new Uint8Array(fileBytes)),
        from: formatMap[fromFormat.value],
        to: formatMap[toFormat.value],
    });

    const bytesToSave = convertedBytes instanceof Uint8Array ? convertedBytes : new Uint8Array(convertedBytes);
    await saveFileWithTauri(bytesToSave, outputName);
    showSuccess(`Saved: ${outputName} — verify on your transmitter before flying!`);
}

async function runBatchConvert() {
    if (!selectedPath) { showError('No input selected'); return; }

    const { invoke } = window.__TAURI__.core;
    const formatMap = { 'edgetx': 'Edgetx', 'ethos': 'Ethos', 'jeti': 'JetiDuplex' };

    const result = await invoke('convert_batch', {
        input_path: selectedPath,
        output_path: deriveOutputPath(selectedPath),
        from: formatMap[fromFormat.value],
        to: 'Edgetx',
    });

    const msg = `Converted ${result.converted} file(s)${result.errors > 0 ? `, ${result.errors} error(s)` : ''}`;
    showSuccess(`${msg} — verify on your transmitter before flying!`);
}

// ============================================================================
// File I/O Helpers
// ============================================================================

function readFileAsBytes(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = (e) => resolve(e.target.result);
        reader.onerror = () => reject(new Error('Failed to read file'));
        reader.readAsArrayBuffer(file);
    });
}

async function saveFileWithTauri(bytes, defaultFileName) {
    const { invoke } = window.__TAURI__.core;
    const { save } = window.__TAURI__.dialog;

    try {
        const savePath = await save({
            defaultPath: defaultFileName,
            filters: [
                { name: 'Model Files', extensions: ['yml', 'bin', 'jsn'] },
                { name: 'All Files', extensions: ['*'] }
            ]
        });

        if (!savePath) throw new Error('Save cancelled');

        await invoke('write_file', { path: savePath, bytes: Array.from(bytes) });
    } catch (error) {
        // Fallback to browser download
        console.warn('Tauri save failed, falling back to browser download:', error);
        const blob = new Blob([bytes], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url; a.download = defaultFileName; a.click();
        URL.revokeObjectURL(url);
    }
}

function deriveOutputPath(inputPath) {
    const sep = inputPath.includes('\\') ? '\\' : '/';
    const parts = inputPath.replace(/[/\\]+$/, '').split(sep);
    parts[parts.length - 1] = parts[parts.length - 1].replace(/\.zip$/i, '') + '_converted';
    return parts.join(sep);
}

function getExtensionForFormat(format) {
    return { edgetx: 'yml', ethos: 'bin', jeti: 'jsn' }[format] || 'bin';
}

function getOutputFileName(originalName, newExt) {
    const parts = originalName.split('.');
    parts.pop();
    return parts.join('.') + '.' + newExt;
}

// ============================================================================
// Status Messages
// ============================================================================

function showSuccess(message) {
    statusMessage.textContent = message;
    statusMessage.className = 'status-message success';
    setTimeout(clearStatus, 5000);
}

function showError(message) {
    statusMessage.textContent = message;
    statusMessage.className = 'status-message error';
}

function showInfo(message) {
    statusMessage.textContent = message;
    statusMessage.className = 'status-message info';
}

function clearStatus() {
    statusMessage.textContent = '';
    statusMessage.className = 'status-message';
}
