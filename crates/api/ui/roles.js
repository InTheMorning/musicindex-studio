// The role menu of a credit and of the usual role of a person (ADR 0112).
// No Preact and no browser state. The list is the role list of the core, as
// `[{ name, roles: [name] }]`. The UI gives only the selected display names.
// The core writes the form of the wire (ADR 0065).

export const ROLE_PLACEHOLDER = 'Select a role';
export const OTHER_ROLE = 'Other role…';
export const OTHER_HINT = 'Use a role from the menu when one fits. Apps know those roles.';
export const NO_ROLE_HINT = 'Apps show a credit with no role as Host.';

const OTHER = 'other';
const norm = (text) => String(text ?? '').trim().toLowerCase();

// The groups of the list that hold the role, in the sequence of the list.
const groupsOf = (role, list) => list.filter((g) => g.roles.some((r) => norm(r) === norm(role)));

// The value of the menu: `group|role` of the list entry, `other` for a role
// that is not of the list, and empty text for no role. A sent group that is
// not a group of the role gives the first group of the role.
export function roleChoice(role, group, list) {
  if (norm(role) === '') return '';
  const groups = groupsOf(role, list);
  if (groups.length === 0) return OTHER;
  const chosen = groups.find((g) => norm(g.name) === norm(group)) || groups[0];
  const name = chosen.roles.find((r) => norm(r) === norm(role));
  return `${chosen.name}|${name}`;
}

// The role and the group, with their display names, for a value of the menu.
// It gives `null` for `other` and for no value.
export function fromChoice(value, list) {
  if (!value || value === OTHER) return null;
  const at = value.indexOf('|');
  if (at < 0) return null;
  const group = value.slice(0, at);
  const role = value.slice(at + 1);
  return list.some((g) => g.name === group && g.roles.includes(role)) ? { role, group } : null;
}

// The options of the menu: one entry for each group, then "Other role…".
export function roleOptions(list) {
  return {
    groups: list.map((g) => ({ label: g.name, options: g.roles.map((r) => [`${g.name}|${r}`, r]) })),
    other: [OTHER, OTHER_ROLE],
  };
}

// The text of a role and a group for a summary line.
export function roleSummary(role, group, list) {
  const choice = roleChoice(role, group, list);
  const picked = fromChoice(choice, list);
  if (picked) return `${picked.role} · ${picked.group}`;
  return [String(role ?? '').trim(), String(group ?? '').trim()].filter(Boolean).join(' · ');
}
