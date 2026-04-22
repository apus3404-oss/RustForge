import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import Settings from './Settings.svelte';
import { api } from '$lib/api';

vi.mock('$lib/api', () => ({
  api: {
    getConfig: vi.fn(),
    updateConfig: vi.fn(),
  },
}));

describe('Settings Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(window, 'alert').mockImplementation(() => {});
  });

  it('loads and displays config on mount', async () => {
    const mockConfig = {
      llm: {
        default_provider: 'anthropic',
        fallback_enabled: true,
      },
      execution: {
        max_parallel_agents: 4,
        default_timeout: 300,
      },
      permissions: {
        default_policy: 'prompt' as const,
      },
    };

    vi.mocked(api.getConfig).mockResolvedValue(mockConfig);

    render(Settings);

    expect(screen.getByText('Loading settings...')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText('System Configuration')).toBeInTheDocument();
    });

    expect(screen.getByDisplayValue('anthropic')).toBeInTheDocument();
    expect(screen.getByDisplayValue('4')).toBeInTheDocument();
    expect(screen.getByDisplayValue('300')).toBeInTheDocument();
  });

  it('displays error when config loading fails', async () => {
    vi.mocked(api.getConfig).mockRejectedValue(new Error('Network error'));

    render(Settings);

    await waitFor(() => {
      expect(screen.getByText('Network error')).toBeInTheDocument();
    });
  });

  it('saves config when save button is clicked', async () => {
    const mockConfig = {
      llm: {
        default_provider: 'anthropic',
        fallback_enabled: true,
      },
      execution: {
        max_parallel_agents: 4,
        default_timeout: 300,
      },
      permissions: {
        default_policy: 'prompt' as const,
      },
    };

    vi.mocked(api.getConfig).mockResolvedValue(mockConfig);
    vi.mocked(api.updateConfig).mockResolvedValue();

    render(Settings);

    await waitFor(() => {
      expect(screen.getByText('System Configuration')).toBeInTheDocument();
    });

    const saveButton = screen.getByRole('button', { name: /save settings/i });
    await userEvent.click(saveButton);

    await waitFor(() => {
      expect(api.updateConfig).toHaveBeenCalledWith(mockConfig);
      expect(window.alert).toHaveBeenCalledWith('Settings saved successfully');
    });
  });

  it('updates config values when inputs change', async () => {
    const mockConfig = {
      llm: {
        default_provider: 'anthropic',
        fallback_enabled: true,
      },
      execution: {
        max_parallel_agents: 4,
        default_timeout: 300,
      },
      permissions: {
        default_policy: 'prompt' as const,
      },
    };

    vi.mocked(api.getConfig).mockResolvedValue(mockConfig);
    vi.mocked(api.updateConfig).mockResolvedValue();

    render(Settings);

    await waitFor(() => {
      expect(screen.getByText('System Configuration')).toBeInTheDocument();
    });

    const providerInput = screen.getByDisplayValue('anthropic');
    await userEvent.clear(providerInput);
    await userEvent.type(providerInput, 'openai');

    const saveButton = screen.getByRole('button', { name: /save settings/i });
    await userEvent.click(saveButton);

    await waitFor(() => {
      expect(api.updateConfig).toHaveBeenCalledWith({
        ...mockConfig,
        llm: {
          ...mockConfig.llm,
          default_provider: 'openai',
        },
      });
    });
  });

  it('displays error when save fails', async () => {
    const mockConfig = {
      llm: {
        default_provider: 'anthropic',
        fallback_enabled: true,
      },
      execution: {
        max_parallel_agents: 4,
        default_timeout: 300,
      },
      permissions: {
        default_policy: 'prompt' as const,
      },
    };

    vi.mocked(api.getConfig).mockResolvedValue(mockConfig);
    vi.mocked(api.updateConfig).mockRejectedValue(new Error('Save failed'));

    render(Settings);

    await waitFor(() => {
      expect(screen.getByText('System Configuration')).toBeInTheDocument();
    });

    const saveButton = screen.getByRole('button', { name: /save settings/i });
    await userEvent.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText('Save failed')).toBeInTheDocument();
    });
  });

  it('disables save button while saving', async () => {
    const mockConfig = {
      llm: {
        default_provider: 'anthropic',
        fallback_enabled: true,
      },
      execution: {
        max_parallel_agents: 4,
        default_timeout: 300,
      },
      permissions: {
        default_policy: 'prompt' as const,
      },
    };

    vi.mocked(api.getConfig).mockResolvedValue(mockConfig);
    vi.mocked(api.updateConfig).mockImplementation(
      () => new Promise((resolve) => setTimeout(resolve, 100))
    );

    render(Settings);

    await waitFor(() => {
      expect(screen.getByText('System Configuration')).toBeInTheDocument();
    });

    const saveButton = screen.getByRole('button', { name: /save settings/i });
    await userEvent.click(saveButton);

    expect(screen.getByRole('button', { name: /saving/i })).toBeDisabled();

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /save settings/i })).not.toBeDisabled();
    });
  });
});
