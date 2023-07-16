<template>
  <div class="flex bg-emerald-900 py-3 px-3 gap-2">
    <NavLink
      to="/"
      text="Home"
    />
    <NavLink
      to="/about"
      text="About"
    />
    <NavLink
      to="/"
      text="Contact TODO"
    />
  </div>
</template>
<script setup lang="ts">
import NavLink from './NavLink.vue';
</script>