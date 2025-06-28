export interface SecurityScanResult {
    safe: boolean;
    risks: SecurityRisk[];
    recommendations: string[];
    score: number; // 0-100, where 100 is safest
}

export interface SecurityRisk {
    type: 'high' | 'medium' | 'low';
    category: string;
    description: string;
    mitigation?: string;
}

export class SecurityScanner {
    private readonly DANGEROUS_IMPORTS = new Set([
        'fs',
        'process',
        'child_process',
        'net',
        'dgram',
        'cluster',
        'http',
        'https',
        'os',
        'v8',
        'vm',
        'worker_threads'
    ]);

    private readonly SAFE_WASI_IMPORTS = new Set([
        'wasi_snapshot_preview1',
        'wasi_snapshot_preview2',
        'wasi:filesystem',
        'wasi:sockets',
        'wasi:clocks',
        'wasi:random',
        'wasi:poll',
        'wasi:io'
    ]);

    private readonly SUSPICIOUS_PATTERNS = [
        /eval/i,
        /exec/i,
        /shell/i,
        /system/i,
        /spawn/i,
        /fork/i,
        /ptrace/i,
        /dlopen/i,
        /mmap/i,
        /mprotect/i
    ];

    async scanComponent(
        wasmBytes: ArrayBuffer,
        imports: WebAssembly.ModuleImportDescriptor[],
        exports: WebAssembly.ModuleExportDescriptor[]
    ): Promise<SecurityScanResult> {
        const risks: SecurityRisk[] = [];
        const recommendations: string[] = [];
        let score = 100;

        // 1. Scan imports for dangerous modules
        this.scanImports(imports, risks, recommendations);

        // 2. Scan exports for suspicious patterns
        this.scanExports(exports, risks, recommendations);

        // 3. Check binary for suspicious byte patterns
        await this.scanBinary(wasmBytes, risks, recommendations);

        // 4. Check for capability requirements
        this.checkCapabilities(imports, risks, recommendations);

        // 5. Memory safety checks
        this.checkMemorySafety(imports, exports, risks, recommendations);

        // Calculate security score
        score = this.calculateSecurityScore(risks);

        return {
            safe: risks.filter(r => r.type === 'high').length === 0,
            risks,
            recommendations,
            score
        };
    }

    private scanImports(
        imports: WebAssembly.ModuleImportDescriptor[],
        risks: SecurityRisk[],
        recommendations: string[]
    ): void {
        const importModules = new Set(imports.map(imp => imp.module));

        importModules.forEach(module => {
            if (this.DANGEROUS_IMPORTS.has(module)) {
                risks.push({
                    type: 'high',
                    category: 'dangerous_import',
                    description: `Imports from dangerous module '${module}' which could access system resources`,
                    mitigation: `Remove dependency on '${module}' or run in a restricted sandbox`
                });
            } else if (!this.SAFE_WASI_IMPORTS.has(module) && !module.startsWith('env')) {
                risks.push({
                    type: 'medium',
                    category: 'unknown_import',
                    description: `Imports from unknown module '${module}'`,
                    mitigation: 'Verify that this import is from a trusted source'
                });
            }
        });

        // Check for filesystem access
        if (importModules.has('wasi:filesystem') || importModules.has('wasi_snapshot_preview1')) {
            const fsImports = imports.filter(imp => 
                imp.module === 'wasi:filesystem' || 
                (imp.module === 'wasi_snapshot_preview1' && imp.name.includes('fd_'))
            );

            if (fsImports.length > 0) {
                risks.push({
                    type: 'medium',
                    category: 'filesystem_access',
                    description: 'Component requires filesystem access',
                    mitigation: 'Ensure filesystem access is restricted to allowed paths only'
                });
                recommendations.push('Consider using a virtual filesystem or restricting paths');
            }
        }

        // Check for network access
        if (importModules.has('wasi:sockets')) {
            risks.push({
                type: 'medium',
                category: 'network_access',
                description: 'Component requires network access',
                mitigation: 'Ensure network access is restricted to allowed endpoints'
            });
            recommendations.push('Consider using a network proxy or firewall rules');
        }
    }

    private scanExports(
        exports: WebAssembly.ModuleExportDescriptor[],
        risks: SecurityRisk[],
        recommendations: string[]
    ): void {
        exports.forEach(exp => {
            // Check for suspicious export names
            for (const pattern of this.SUSPICIOUS_PATTERNS) {
                if (pattern.test(exp.name)) {
                    risks.push({
                        type: 'medium',
                        category: 'suspicious_export',
                        description: `Export '${exp.name}' matches suspicious pattern '${pattern}'`,
                        mitigation: 'Review the exported function to ensure it doesn\'t perform unsafe operations'
                    });
                }
            }

            // Check for raw memory exports
            if (exp.kind === 'memory') {
                risks.push({
                    type: 'low',
                    category: 'memory_export',
                    description: 'Component exports its memory, which could be accessed by host',
                    mitigation: 'Ensure sensitive data is properly encrypted or cleared'
                });
                recommendations.push('Consider not exporting memory unless necessary');
            }

            // Check for table exports (indirect calls)
            if (exp.kind === 'table') {
                risks.push({
                    type: 'low',
                    category: 'indirect_calls',
                    description: 'Component exports function table for indirect calls',
                    mitigation: 'Verify that function pointers cannot be manipulated by untrusted code'
                });
            }
        });
    }

    private async scanBinary(
        wasmBytes: ArrayBuffer,
        risks: SecurityRisk[],
        recommendations: string[]
    ): Promise<void> {
        const bytes = new Uint8Array(wasmBytes);
        
        // Check for suspicious byte patterns (simplified)
        const suspiciousStrings = [
            'bash',
            'sh -c',
            'cmd.exe',
            'powershell',
            '/etc/passwd',
            'SELECT * FROM',
            'DROP TABLE',
            '<script>',
            'javascript:',
            '../../../'
        ];

        // Convert bytes to string for pattern matching (naive approach)
        const decoder = new TextDecoder('utf-8', { fatal: false });
        const text = decoder.decode(bytes);

        suspiciousStrings.forEach(pattern => {
            if (text.includes(pattern)) {
                risks.push({
                    type: 'medium',
                    category: 'suspicious_content',
                    description: `Binary contains suspicious string: '${pattern}'`,
                    mitigation: 'Review the component source code for potential security issues'
                });
            }
        });

        // Check for large binary size (could indicate embedded data)
        if (bytes.length > 10 * 1024 * 1024) { // 10MB
            risks.push({
                type: 'low',
                category: 'large_binary',
                description: 'Component binary is unusually large, may contain embedded data',
                mitigation: 'Verify that the size is justified by the component\'s functionality'
            });
        }
    }

    private checkCapabilities(
        imports: WebAssembly.ModuleImportDescriptor[],
        risks: SecurityRisk[],
        recommendations: string[]
    ): void {
        // Group imports by WASI capability
        const capabilities = new Map<string, string[]>();

        imports.forEach(imp => {
            if (imp.module.startsWith('wasi:')) {
                const cap = imp.module.split(':')[1];
                const funcs = capabilities.get(cap) || [];
                funcs.push(imp.name);
                capabilities.set(cap, funcs);
            }
        });

        // Check for dangerous capability combinations
        if (capabilities.has('filesystem') && capabilities.has('sockets')) {
            risks.push({
                type: 'high',
                category: 'capability_combination',
                description: 'Component has both filesystem and network access',
                mitigation: 'This combination could allow exfiltration of local files'
            });
            recommendations.push('Consider splitting into separate components with limited capabilities');
        }

        // Check for process spawning capabilities
        const processCapabilities = ['process', 'threads', 'subprocess'];
        const hasProcessCap = processCapabilities.some(cap => capabilities.has(cap));
        
        if (hasProcessCap) {
            risks.push({
                type: 'high',
                category: 'process_spawning',
                description: 'Component can spawn new processes',
                mitigation: 'This capability is extremely dangerous and should be avoided'
            });
        }
    }

    private checkMemorySafety(
        imports: WebAssembly.ModuleImportDescriptor[],
        exports: WebAssembly.ModuleExportDescriptor[],
        risks: SecurityRisk[],
        recommendations: string[]
    ): void {
        // Check for shared memory
        const hasSharedMemory = imports.some(imp => 
            imp.kind === 'memory' && imp.name.includes('shared')
        ) || exports.some(exp => 
            exp.kind === 'memory' && exp.name.includes('shared')
        );

        if (hasSharedMemory) {
            risks.push({
                type: 'medium',
                category: 'shared_memory',
                description: 'Component uses shared memory which could lead to race conditions',
                mitigation: 'Ensure proper synchronization mechanisms are in place'
            });
            recommendations.push('Consider using message passing instead of shared memory');
        }

        // Check for multiple memories (could indicate complex memory management)
        const memoryCount = imports.filter(imp => imp.kind === 'memory').length +
                           exports.filter(exp => exp.kind === 'memory').length;
        
        if (memoryCount > 1) {
            risks.push({
                type: 'low',
                category: 'multiple_memories',
                description: 'Component uses multiple memory instances',
                mitigation: 'Verify that memory isolation is properly maintained'
            });
        }
    }

    private calculateSecurityScore(risks: SecurityRisk[]): number {
        let score = 100;

        risks.forEach(risk => {
            switch (risk.type) {
                case 'high':
                    score -= 30;
                    break;
                case 'medium':
                    score -= 15;
                    break;
                case 'low':
                    score -= 5;
                    break;
            }
        });

        return Math.max(0, score);
    }

    // Utility method to generate a security report
    generateReport(scanResult: SecurityScanResult): string {
        const report: string[] = [
            '# Security Scan Report',
            '',
            `**Overall Score**: ${scanResult.score}/100`,
            `**Status**: ${scanResult.safe ? '✅ SAFE' : '⚠️ RISKS DETECTED'}`,
            ''
        ];

        if (scanResult.risks.length > 0) {
            report.push('## Risks Identified');
            report.push('');
            
            const risksByType = {
                high: scanResult.risks.filter(r => r.type === 'high'),
                medium: scanResult.risks.filter(r => r.type === 'medium'),
                low: scanResult.risks.filter(r => r.type === 'low')
            };

            if (risksByType.high.length > 0) {
                report.push('### 🔴 High Risk');
                risksByType.high.forEach(risk => {
                    report.push(`- **${risk.category}**: ${risk.description}`);
                    if (risk.mitigation) {
                        report.push(`  - *Mitigation*: ${risk.mitigation}`);
                    }
                });
                report.push('');
            }

            if (risksByType.medium.length > 0) {
                report.push('### 🟡 Medium Risk');
                risksByType.medium.forEach(risk => {
                    report.push(`- **${risk.category}**: ${risk.description}`);
                    if (risk.mitigation) {
                        report.push(`  - *Mitigation*: ${risk.mitigation}`);
                    }
                });
                report.push('');
            }

            if (risksByType.low.length > 0) {
                report.push('### 🟢 Low Risk');
                risksByType.low.forEach(risk => {
                    report.push(`- **${risk.category}**: ${risk.description}`);
                    if (risk.mitigation) {
                        report.push(`  - *Mitigation*: ${risk.mitigation}`);
                    }
                });
                report.push('');
            }
        }

        if (scanResult.recommendations.length > 0) {
            report.push('## Recommendations');
            report.push('');
            scanResult.recommendations.forEach(rec => {
                report.push(`- ${rec}`);
            });
        }

        return report.join('\n');
    }
}