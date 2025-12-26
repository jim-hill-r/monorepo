import * as assert from 'assert';

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from 'vscode';
import * as myExtension from '../extension';

suite('Extension Test Suite', () => {
	vscode.window.showInformationMessage('Start all tests.');

	test('Sample test', () => {
		assert.strictEqual(-1, [1, 2, 3].indexOf(5));
		assert.strictEqual(-1, [1, 2, 3].indexOf(0));
	});

	suite('getSessionStart', () => {
		test('should parse session log with Start entry', () => {
			// This is the format that Cast actually writes (capital S)
			const sessionLog = '2025-01-01 12:00:00 UTC,Start\n';
			const result = myExtension.getSessionStart(sessionLog);
			assert.notStrictEqual(result, undefined, 'Should parse Start entry');
			assert.ok(result instanceof Date, 'Should return a Date object');
		});

		test('should return undefined for empty log', () => {
			const sessionLog = '';
			const result = myExtension.getSessionStart(sessionLog);
			assert.strictEqual(result, undefined, 'Should return undefined for empty log');
		});

		test('should return undefined when no Start entry exists', () => {
			const sessionLog = '2025-01-01 12:00:00 UTC,Pause\n2025-01-01 12:30:00 UTC,Stop\n';
			const result = myExtension.getSessionStart(sessionLog);
			assert.strictEqual(result, undefined, 'Should return undefined when no Start entry exists');
		});

		test('should parse session log with multiple entries', () => {
			const sessionLog = '2025-01-01 12:00:00 UTC,Start\n2025-01-01 12:30:00 UTC,Pause\n2025-01-01 13:00:00 UTC,Start\n';
			const result = myExtension.getSessionStart(sessionLog);
			assert.notStrictEqual(result, undefined, 'Should find first Start entry');
		});

		test('should parse session log with named session', () => {
			const sessionLog = '2025-01-01 12:00:00 UTC,Start,my-session\n';
			const result = myExtension.getSessionStart(sessionLog);
			assert.notStrictEqual(result, undefined, 'Should parse Start entry with name');
			assert.ok(result instanceof Date, 'Should return a Date object');
		});
	});
});
