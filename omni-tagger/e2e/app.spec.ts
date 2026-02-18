import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mockInvoke = async (cmd: string, args: any) => {
      console.log(`[Mock Invoke] ${cmd}`, args);

      if (cmd.startsWith('plugin:event|')) {
        return 123; // Event ID or similar
      }

      if (cmd === 'get_config') {
        return {
          model_path: 'models/model.onnx',
          tags_path: 'models/tags.csv',
          threshold: 0.5,
          use_underscore: true,
          exclusion_list: ['nsfw', 'monochrome'],
        };
      }
      if (cmd === 'check_model_exists') {
        return true;
      }
      if (cmd === 'register_context_menu') {
        return;
      }

      console.warn(`Unhandled mock command: ${cmd}`);
      return null;
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__TAURI_INTERNALS__ = {
      invoke: mockInvoke,
      plugins: {
        invoke: mockInvoke,
      },
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__TAURI__ = {
      invoke: mockInvoke,
      event: {
        listen: async () => {
          return () => {};
        },
      },
    };
  });
});

test('has settings header', async ({ page }) => {
  await page.goto('/');

  await expect(
    page.getByRole('heading', { name: 'OmniTagger Settings' })
  ).toBeVisible();
});

test('loads configuration correctly', async ({ page }) => {
  await page.goto('/');

  // Verify threshold
  await expect(page.getByText('Confidence Threshold: 0.5')).toBeVisible();

  // Verify exclusion list
  await expect(page.locator('textarea')).toHaveValue('nsfw, monochrome');
});
