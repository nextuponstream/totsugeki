export function nameFallback(name: string) {
  if (name.length === 0) {
    return 'unnamed'
  } else {
    return name
  }
}
