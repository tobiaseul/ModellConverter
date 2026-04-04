// UI Elements
const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('file-input');
const browseBtn = document.getElementById('browse-btn');
const fromFormat = document.getElementById('from-format');
const toFormat = document.getElementById('to-format');
const convertBtn = document.getElementById('convert-btn');
const statusMessage = document.getElementById('status-message');

// State
let selectedFile = null;

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    setupDragAndDrop();
    setupBrowseButton();
    setupFormatSelectors();
    setupConvertButton();
});

// ============================================================================
// Drag and Drop
// ============================================================================

function setupDragAndDrop() {
    dropZone.addEventListener('dragover', handleDragOver);
    dropZone.addEventListener('dragleave', handleDragLeave);
    dropZone.addEventListener('drop', handleDrop);
    dropZone.addEventListener('click', () => fileInput.click());
}

function handleDragOver(e) {
    e.preventDefault();
    e.stopPropagation();
    dropZone.classList.add('drag-over');
}

function handleDragLeave(e) {
    e.preventDefault();
    e.stopPropagation();
    dropZone.classList.remove('drag-over');
}

function handleDrop(e) {
    e.preventDefault();
    e.stopPropagation();
    dropZone.classList.remove('drag-over');

    const files = e.dataTransfer.files;
    if (files.length > 0) {
        selectFile(files[0]);
    }
}

// ============================================================================
// File Selection
// ============================================================================

function setupBrowseButton() {
    browseBtn.addEventListener('click', (e) => {
        e.preventDefault();
        fileInput.click();
    });

    fileInput.addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
            selectFile(e.target.files[0]);
        }
    });
}

function selectFile(file) {
    selectedFile = file;

    // Update UI to show file selected
    const dropZoneContent = dropZone.querySelector('.drop-zone-content');
    dropZoneContent.innerHTML = `
        <svg class="drop-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <p class="drop-text">${file.name}</p>
        <p class="drop-subtext"><a href="#" onclick="event.preventDefault(); document.getElementById('file-input').click();">Change file</a></p>
    `;

    updateConvertButtonState();
    clearStatus();
}

// ============================================================================
// Format Selectors
// ============================================================================

function setupFormatSelectors() {
    fromFormat.addEventListener('change', updateConvertButtonState);
    // Target format is fixed to EdgeTX
}

// ============================================================================
// Convert Button
// ============================================================================

function setupConvertButton() {
    convertBtn.addEventListener('click', handleConvert);
}

function updateConvertButtonState() {
    const canConvert = selectedFile !== null;
    convertBtn.disabled = !canConvert;
}

async function handleConvert() {
    if (!selectedFile) {
        showError('No file selected');
        return;
    }

    convertBtn.classList.add('loading');
    showInfo('Converting...');

    try {
        // Read file as bytes
        const fileBytes = await readFileAsBytes(selectedFile);

        // Get target extension based on format
        const targetExt = getExtensionForFormat(toFormat.value);
        const outputName = getOutputFileName(selectedFile.name, targetExt);

        // Parse format enums to match Rust Format enum
        const formatMap = {
            'edgetx': 'Edgetx',
            'ethos': 'Ethos',
            'jeti': 'JetiDuplex'
        };

        // Call Tauri command to convert
        const { invoke } = window.__TAURI__.core;
        const convertedBytes = await invoke('convert_model', {
            input_bytes: Array.from(new Uint8Array(fileBytes)),
            from: formatMap[fromFormat.value],
            to: formatMap[toFormat.value]
        });

        // Convert response back to Uint8Array if needed
        const bytesToSave = convertedBytes instanceof Uint8Array
            ? convertedBytes
            : new Uint8Array(convertedBytes);

        // Save the file using Tauri dialog
        await saveFileWithTauri(bytesToSave, outputName);

        showSuccess(`✓ Saved: ${outputName} — Verify on your transmitter before flying!`);
    } catch (error) {
        showError(`Error: ${error.message}`);
    } finally {
        convertBtn.classList.remove('loading');
    }
}

function readFileAsBytes(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = (e) => resolve(e.target.result);
        reader.onerror = () => reject(new Error('Failed to read file'));
        reader.readAsArrayBuffer(file);
    });
}

async function saveFileWithTauri(bytes, defaultFileName) {
    // Use Tauri's save dialog
    const { invoke } = window.__TAURI__.core;
    const { save } = window.__TAURI__.dialog;

    try {
        // Show save dialog
        const savePath = await save({
            defaultPath: defaultFileName,
            filters: [
                { name: 'Model Files', extensions: ['yml', 'bin', 'jsn'] },
                { name: 'All Files', extensions: ['*'] }
            ]
        });

        if (!savePath) {
            // User cancelled
            throw new Error('Save cancelled');
        }

        // Write file using Tauri command
        await invoke('write_file', {
            path: savePath,
            bytes: Array.from(bytes)
        });
    } catch (error) {
        // Fallback to browser download if Tauri fails
        console.warn('Tauri save failed, falling back to browser download:', error);
        const blob = new Blob([bytes], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = defaultFileName;
        a.click();
        URL.revokeObjectURL(url);
    }
}

function getExtensionForFormat(format) {
    const extensions = {
        edgetx: 'yml',
        ethos: 'bin',
        jeti: 'jsn'
    };
    return extensions[format] || 'bin';
}

function getOutputFileName(originalName, newExt) {
    const parts = originalName.split('.');
    parts.pop(); // Remove old extension
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
