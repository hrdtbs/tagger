import { test, expect } from '@playwright/test';

// Define the interface for the mock window
interface MockWindow extends Window {
  __TAURI_INTERNALS__: {
    invoke: (cmd: string, args: unknown) => Promise<unknown>;
    plugins: {
      invoke: (cmd: string, args: unknown) => Promise<unknown>;
    };
  };
  __TAURI__: {
    invoke: (cmd: string, args: unknown) => Promise<unknown>;
    event: {
      listen: (event: string, handler: unknown) => Promise<() => void>;
    };
  };
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const mockInvoke = async (cmd: string, args: unknown) => {
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

    const mockWindow = window as unknown as MockWindow;

    mockWindow.__TAURI_INTERNALS__ = {
      invoke: mockInvoke,
      plugins: {
        invoke: mockInvoke,
      },
    };

    mockWindow.__TAURI__ = {
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

  await expect(page.getByRole('heading', { name: 'OmniTagger Settings' })).toBeVisible();
});

test('loads configuration correctly', async ({ page }) => {
  await page.goto('/');

  // Verify threshold
  await expect(page.getByText('Confidence Threshold: 0.5')).toBeVisible();

  // Verify exclusion list
  await expect(page.locator('textarea')).toHaveValue('nsfw, monochrome');
});
