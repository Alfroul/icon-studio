<script setup lang="ts">
import { useUiStore } from "@/stores/ui";

const ui = useUiStore();
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div
          v-for="toast in ui.toasts"
          :key="toast.id"
          :class="['toast', `toast-${toast.type}`]"
          role="alert"
          aria-live="assertive"
        >
          {{ toast.text }}
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 36px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  pointer-events: auto;
  backdrop-filter: blur(12px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.06);
}

.toast-info {
  background: var(--bg-glass);
  border: 1px solid var(--accent);
  color: var(--accent);
}

.toast-success {
  background: var(--bg-glass);
  border: 1px solid var(--success);
  color: var(--success);
}

.toast-error {
  background: var(--bg-glass);
  border: 1px solid var(--danger);
  color: var(--danger);
}

.toast-warning {
  background: var(--warning-muted);
  border: 1px solid var(--warning);
  color: var(--warning);
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(40px);
}
</style>
