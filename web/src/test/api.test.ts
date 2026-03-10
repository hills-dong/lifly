import { describe, it, expect } from 'vitest';
import { auth, tools, dataObjects, rawInputs, pipelines, reminders, categories, files } from '../api';

describe('API module exports', () => {
  it('exports auth functions', () => {
    expect(auth.login).toBeTypeOf('function');
    expect(auth.logout).toBeTypeOf('function');
  });

  it('exports tools functions', () => {
    expect(tools.listTools).toBeTypeOf('function');
    expect(tools.getTool).toBeTypeOf('function');
  });

  it('exports dataObjects functions', () => {
    expect(dataObjects.listDataObjects).toBeTypeOf('function');
    expect(dataObjects.getDataObject).toBeTypeOf('function');
    expect(dataObjects.updateDataObject).toBeTypeOf('function');
    expect(dataObjects.deleteDataObject).toBeTypeOf('function');
    expect(dataObjects.searchDataObjects).toBeTypeOf('function');
  });

  it('exports rawInputs functions', () => {
    expect(rawInputs.createRawInput).toBeTypeOf('function');
  });

  it('exports pipelines functions', () => {
    expect(pipelines.listPipelines).toBeTypeOf('function');
    expect(pipelines.getPipeline).toBeTypeOf('function');
  });

  it('exports reminders functions', () => {
    expect(reminders.listReminders).toBeTypeOf('function');
    expect(reminders.createReminder).toBeTypeOf('function');
    expect(reminders.updateReminder).toBeTypeOf('function');
    expect(reminders.deleteReminder).toBeTypeOf('function');
    expect(reminders.dismissReminder).toBeTypeOf('function');
  });

  it('exports categories functions', () => {
    expect(categories.listCategories).toBeTypeOf('function');
    expect(categories.createCategory).toBeTypeOf('function');
    expect(categories.updateCategory).toBeTypeOf('function');
    expect(categories.deleteCategory).toBeTypeOf('function');
  });

  it('exports files functions', () => {
    expect(files.uploadFile).toBeTypeOf('function');
    expect(files.getFileUrl).toBeTypeOf('function');
  });

  it('generates correct file URLs', () => {
    expect(files.getFileUrl('abc-123')).toBe('/api/files/abc-123');
  });
});
