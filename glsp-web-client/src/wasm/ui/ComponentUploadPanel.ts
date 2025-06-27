import { WasmTranspiler } from '../transpiler/WasmTranspiler.js';
import { ComponentRegistry } from '../runtime/ComponentRegistry.js';

export interface UploadProgress {
    stage: 'uploading' | 'validating' | 'transpiling' | 'registering' | 'complete' | 'error';
    progress: number; // 0-100
    message: string;
    error?: string;
}

export class ComponentUploadPanel {
    private element: HTMLElement;
    private transpiler: WasmTranspiler;
    private registry: ComponentRegistry;
    private onUploadComplete?: (componentId: string) => void;
    private onUploadError?: (error: string) => void;

    constructor(
        transpiler: WasmTranspiler, 
        registry: ComponentRegistry,
        onUploadComplete?: (componentId: string) => void,
        onUploadError?: (error: string) => void
    ) {
        this.transpiler = transpiler;
        this.registry = registry;
        this.onUploadComplete = onUploadComplete;
        this.onUploadError = onUploadError;
        this.element = this.createElement();
        this.setupEventHandlers();
    }

    private createElement(): HTMLElement {
        const panel = document.createElement('div');
        panel.className = 'component-upload-panel';
        panel.style.cssText = `
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            width: 500px;
            background: var(--bg-secondary, #151B2C);
            border: 1px solid var(--border-color, #2A3441);
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
            z-index: 1000;
            display: none;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            color: var(--text-primary, #E5E9F0);
        `;

        panel.innerHTML = `
            <div class="upload-header">
                <h3 style="margin: 0 0 15px 0; color: var(--text-primary, #E5E9F0);">Upload WASM Component</h3>
                <button class="close-btn" style="
                    position: absolute;
                    top: 15px;
                    right: 15px;
                    background: none;
                    border: none;
                    color: var(--text-secondary, #A0A9BA);
                    font-size: 18px;
                    cursor: pointer;
                    padding: 5px;
                ">×</button>
            </div>

            <div class="upload-area" style="
                border: 2px dashed var(--border-color, #2A3441);
                border-radius: 8px;
                padding: 40px 20px;
                text-align: center;
                margin-bottom: 20px;
                transition: all 0.3s ease;
                cursor: pointer;
            ">
                <div class="upload-icon" style="
                    font-size: 48px;
                    color: var(--text-secondary, #A0A9BA);
                    margin-bottom: 15px;
                ">📁</div>
                <div class="upload-text" style="
                    color: var(--text-secondary, #A0A9BA);
                    margin-bottom: 10px;
                ">Drop WASM file here or click to browse</div>
                <div class="upload-hint" style="
                    font-size: 12px;
                    color: var(--text-tertiary, #6B7280);
                ">Supports .wasm files up to 50MB</div>
                <input type="file" accept=".wasm" style="display: none;" />
            </div>

            <div class="component-info" style="display: none; margin-bottom: 20px;">
                <div class="info-row" style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                    <span style="color: var(--text-secondary, #A0A9BA);">File:</span>
                    <span class="file-name" style="color: var(--text-primary, #E5E9F0);"></span>
                </div>
                <div class="info-row" style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                    <span style="color: var(--text-secondary, #A0A9BA);">Size:</span>
                    <span class="file-size" style="color: var(--text-primary, #E5E9F0);"></span>
                </div>
                <div class="name-input-row" style="margin-top: 15px;">
                    <label style="display: block; color: var(--text-secondary, #A0A9BA); margin-bottom: 5px;">Component Name:</label>
                    <input type="text" class="component-name-input" placeholder="Enter component name..." style="
                        width: 100%;
                        padding: 8px 12px;
                        background: var(--bg-primary, #0F1419);
                        border: 1px solid var(--border-color, #2A3441);
                        border-radius: 4px;
                        color: var(--text-primary, #E5E9F0);
                        font-size: 14px;
                        box-sizing: border-box;
                    " />
                </div>
            </div>

            <div class="progress-section" style="display: none; margin-bottom: 20px;">
                <div class="progress-text" style="
                    color: var(--text-primary, #E5E9F0);
                    margin-bottom: 8px;
                    font-size: 14px;
                "></div>
                <div class="progress-bar-container" style="
                    background: var(--bg-primary, #0F1419);
                    border-radius: 10px;
                    height: 20px;
                    overflow: hidden;
                ">
                    <div class="progress-bar" style="
                        background: linear-gradient(90deg, #4A9EFF, #00D4AA);
                        height: 100%;
                        width: 0%;
                        transition: width 0.3s ease;
                        border-radius: 10px;
                    "></div>
                </div>
                <div class="progress-details" style="
                    font-size: 12px;
                    color: var(--text-tertiary, #6B7280);
                    margin-top: 5px;
                "></div>
            </div>

            <div class="error-section" style="display: none; margin-bottom: 20px;">
                <div class="error-message" style="
                    background: rgba(239, 68, 68, 0.1);
                    border: 1px solid rgba(239, 68, 68, 0.3);
                    border-radius: 6px;
                    padding: 12px;
                    color: #FCA5A5;
                    font-size: 14px;
                "></div>
            </div>

            <div class="action-buttons" style="
                display: flex;
                gap: 10px;
                justify-content: flex-end;
            ">
                <button class="cancel-btn" style="
                    padding: 8px 16px;
                    background: var(--bg-primary, #0F1419);
                    border: 1px solid var(--border-color, #2A3441);
                    border-radius: 4px;
                    color: var(--text-secondary, #A0A9BA);
                    cursor: pointer;
                    font-size: 14px;
                ">Cancel</button>
                <button class="upload-btn" style="
                    padding: 8px 16px;
                    background: linear-gradient(90deg, #4A9EFF, #00D4AA);
                    border: none;
                    border-radius: 4px;
                    color: white;
                    cursor: pointer;
                    font-size: 14px;
                    font-weight: 500;
                    disabled: true;
                " disabled>Upload & Transpile</button>
            </div>
        `;

        return panel;
    }

    private setupEventHandlers(): void {
        const uploadArea = this.element.querySelector('.upload-area') as HTMLElement;
        const fileInput = this.element.querySelector('input[type="file"]') as HTMLInputElement;
        const closeBtn = this.element.querySelector('.close-btn') as HTMLButtonElement;
        const cancelBtn = this.element.querySelector('.cancel-btn') as HTMLButtonElement;
        const uploadBtn = this.element.querySelector('.upload-btn') as HTMLButtonElement;
        const componentNameInput = this.element.querySelector('.component-name-input') as HTMLInputElement;

        // File drop and click handlers
        uploadArea.addEventListener('click', () => fileInput.click());
        uploadArea.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadArea.style.borderColor = 'var(--accent-color, #4A9EFF)';
            uploadArea.style.backgroundColor = 'rgba(74, 158, 255, 0.05)';
        });
        uploadArea.addEventListener('dragleave', (e) => {
            e.preventDefault();
            uploadArea.style.borderColor = 'var(--border-color, #2A3441)';
            uploadArea.style.backgroundColor = 'transparent';
        });
        uploadArea.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadArea.style.borderColor = 'var(--border-color, #2A3441)';
            uploadArea.style.backgroundColor = 'transparent';

            const files = e.dataTransfer?.files;
            if (files && files.length > 0) {
                this.handleFileSelection(files[0]);
            }
        });

        fileInput.addEventListener('change', (e) => {
            const files = (e.target as HTMLInputElement).files;
            if (files && files.length > 0) {
                this.handleFileSelection(files[0]);
            }
        });

        // Button handlers
        closeBtn.addEventListener('click', () => this.hide());
        cancelBtn.addEventListener('click', () => this.hide());
        uploadBtn.addEventListener('click', () => this.handleUpload());

        // Enable upload button when name is entered
        componentNameInput.addEventListener('input', () => {
            uploadBtn.disabled = !componentNameInput.value.trim();
        });
    }

    private handleFileSelection(file: File): void {
        if (!file.name.endsWith('.wasm')) {
            this.showError('Please select a valid .wasm file');
            return;
        }

        if (file.size > 50 * 1024 * 1024) {
            this.showError('File too large. Maximum size is 50MB');
            return;
        }

        // Show file info
        const componentInfo = this.element.querySelector('.component-info') as HTMLElement;
        const fileName = this.element.querySelector('.file-name') as HTMLElement;
        const fileSize = this.element.querySelector('.file-size') as HTMLElement;
        const componentNameInput = this.element.querySelector('.component-name-input') as HTMLInputElement;

        fileName.textContent = file.name;
        fileSize.textContent = this.formatFileSize(file.size);
        
        // Auto-generate component name from filename
        const baseName = file.name.replace('.wasm', '').replace(/[^a-zA-Z0-9-_]/g, '-');
        componentNameInput.value = baseName;
        
        componentInfo.style.display = 'block';
        
        // Store file for upload
        (this.element as any)._selectedFile = file;
        
        // Enable upload button
        const uploadBtn = this.element.querySelector('.upload-btn') as HTMLButtonElement;
        uploadBtn.disabled = false;

        // Hide error if showing
        this.hideError();
    }

    private async handleUpload(): Promise<void> {
        const file = (this.element as any)._selectedFile as File;
        const componentNameInput = this.element.querySelector('.component-name-input') as HTMLInputElement;
        const componentName = componentNameInput.value.trim();

        if (!file || !componentName) {
            this.showError('Please select a file and enter a component name');
            return;
        }

        this.showProgress({ stage: 'uploading', progress: 0, message: 'Reading file...' });

        try {
            // Read file
            const arrayBuffer = await this.readFileAsArrayBuffer(file);
            this.updateProgress({ stage: 'validating', progress: 25, message: 'Validating component...' });

            // Transpile component
            this.updateProgress({ stage: 'transpiling', progress: 50, message: 'Transpiling to JavaScript...' });
            const transpiledComponent = await this.transpiler.transpileComponent(arrayBuffer, componentName);

            // Register component
            this.updateProgress({ stage: 'registering', progress: 75, message: 'Registering component...' });
            const componentId = this.registry.registerComponent(transpiledComponent);

            // Complete
            this.updateProgress({ stage: 'complete', progress: 100, message: 'Component uploaded successfully!' });

            setTimeout(() => {
                this.onUploadComplete?.(componentId);
                this.hide();
            }, 1500);

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
            this.showError(errorMessage);
            this.onUploadError?.(errorMessage);
        }
    }

    private readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result as ArrayBuffer);
            reader.onerror = () => reject(new Error('Failed to read file'));
            reader.readAsArrayBuffer(file);
        });
    }

    private showProgress(progress: UploadProgress): void {
        const progressSection = this.element.querySelector('.progress-section') as HTMLElement;
        progressSection.style.display = 'block';
        this.updateProgress(progress);

        // Hide other sections
        this.element.querySelector('.component-info')!.setAttribute('style', 'display: none !important');
        this.element.querySelector('.action-buttons')!.setAttribute('style', 'display: none !important');
        this.hideError();
    }

    private updateProgress(progress: UploadProgress): void {
        const progressText = this.element.querySelector('.progress-text') as HTMLElement;
        const progressBar = this.element.querySelector('.progress-bar') as HTMLElement;
        const progressDetails = this.element.querySelector('.progress-details') as HTMLElement;

        progressText.textContent = progress.message;
        progressBar.style.width = `${progress.progress}%`;
        progressDetails.textContent = `${progress.stage} (${progress.progress}%)`;
    }

    private showError(message: string): void {
        const errorSection = this.element.querySelector('.error-section') as HTMLElement;
        const errorMessage = this.element.querySelector('.error-message') as HTMLElement;
        
        errorMessage.textContent = message;
        errorSection.style.display = 'block';
    }

    private hideError(): void {
        const errorSection = this.element.querySelector('.error-section') as HTMLElement;
        errorSection.style.display = 'none';
    }

    private formatFileSize(bytes: number): string {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    }

    show(): void {
        this.element.style.display = 'block';
        document.body.appendChild(this.element);
    }

    hide(): void {
        this.element.style.display = 'none';
        
        // Reset form
        const componentInfo = this.element.querySelector('.component-info') as HTMLElement;
        const progressSection = this.element.querySelector('.progress-section') as HTMLElement;
        const actionButtons = this.element.querySelector('.action-buttons') as HTMLElement;
        const componentNameInput = this.element.querySelector('.component-name-input') as HTMLInputElement;
        const uploadBtn = this.element.querySelector('.upload-btn') as HTMLButtonElement;

        componentInfo.style.display = 'none';
        progressSection.style.display = 'none';
        actionButtons.style.display = 'flex';
        componentNameInput.value = '';
        uploadBtn.disabled = true;
        
        this.hideError();
        (this.element as any)._selectedFile = null;
    }

    getElement(): HTMLElement {
        return this.element;
    }
}