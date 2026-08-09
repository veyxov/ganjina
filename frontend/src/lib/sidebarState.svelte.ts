let collapsed = $state(localStorage.getItem('sidebar-collapsed') === '1');

export function sidebarCollapsed() {
  return collapsed;
}

export function toggleSidebar() {
  collapsed = !collapsed;
  localStorage.setItem('sidebar-collapsed', collapsed ? '1' : '0');
}
