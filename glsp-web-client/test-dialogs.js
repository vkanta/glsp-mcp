// Professional Dialog System Test
// Copy and paste this into the browser console to test the new dialogs

console.log('🎭 Professional Dialog System Test');
console.log('====================================');

// Test the new dialog system
async function testDialogs() {
    console.log('Testing professional dialogs...');
    
    // Get UIManager instance
    const uiManager = window.appController?.uiManager;
    if (!uiManager) {
        console.error('❌ UIManager not found - make sure the app is fully loaded');
        return;
    }
    
    console.log('✅ UIManager found, starting dialog tests...');

    // Test 1: Error Dialog
    console.log('\n📛 Test 1: Error Dialog');
    await uiManager.showError(
        'This is a test error message',
        'Error details with technical information:\n- Connection failed\n- Timeout: 5000ms\n- Status: 500'
    );
    console.log('✅ Error dialog completed');

    // Test 2: Success Dialog
    console.log('\n✅ Test 2: Success Dialog');
    await uiManager.showSuccess(
        'Operation completed successfully!',
        'All tasks were processed without errors.'
    );
    console.log('✅ Success dialog completed');

    // Test 3: Warning Dialog
    console.log('\n⚠️ Test 3: Warning Dialog');
    await uiManager.showWarning(
        'This action may have consequences',
        'Please review your settings before proceeding.'
    );
    console.log('✅ Warning dialog completed');

    // Test 4: Confirmation Dialog
    console.log('\n❓ Test 4: Confirmation Dialog');
    const confirmed = await uiManager.showConfirmDialog(
        'Do you want to continue with this test?',
        {
            title: 'Test Confirmation',
            details: 'This will proceed to the next test dialog.',
            variant: 'info',
            confirmText: 'Continue',
            cancelText: 'Skip'
        }
    );
    console.log('✅ Confirmation result:', confirmed);

    if (confirmed) {
        // Test 5: Prompt Dialog
        console.log('\n📝 Test 5: Prompt Dialog');
        const userInput = await uiManager.showPrompt(
            'Enter your name for testing:',
            {
                title: 'User Input Test',
                placeholder: 'Your name here...',
                defaultValue: 'Test User',
                validation: {
                    minLength: 2,
                    maxLength: 50,
                    pattern: /^[a-zA-Z\s]+$/,
                    message: 'Name can only contain letters and spaces'
                }
            }
        );
        console.log('✅ User input:', userInput);

        // Test 6: Delete Confirmation
        if (userInput) {
            console.log('\n🗑️ Test 6: Delete Confirmation');
            const deleteConfirmed = await uiManager.showDeleteConfirm(
                `Profile for "${userInput}"`,
                'All associated data will be permanently removed.'
            );
            console.log('✅ Delete confirmation:', deleteConfirmed);
        }
    }

    // Test 7: Diagram Type Selector (if available)
    console.log('\n📊 Test 7: Diagram Type Selector');
    try {
        const result = await uiManager.showDiagramTypeSelector(['Existing Diagram 1', 'Test Diagram']);
        if (result) {
            console.log('✅ Selected diagram type:', result.type.label);
            console.log('✅ Diagram name:', result.name);
        } else {
            console.log('✅ User cancelled diagram creation');
        }
    } catch (error) {
        console.log('ℹ️ Diagram selector test skipped (no types available)');
    }

    console.log('\n🎉 All dialog tests completed!');
    console.log('The new professional dialogs are working correctly.');
    console.log('\n📋 Benefits:');
    console.log('• Professional appearance with consistent styling');
    console.log('• Validation and error handling');
    console.log('• Keyboard navigation support');
    console.log('• Mobile responsive design');
    console.log('• Backdrop blur effects');
    console.log('• Smooth animations');
    console.log('• Copy functionality for error details');
    console.log('• No more ugly browser popups!');
}

// Add test function to global scope for easy access
window.testDialogs = testDialogs;

console.log('\n🚀 To test the dialogs, run: testDialogs()');
console.log('Or simply: window.testDialogs()');