data:extend({
  {type = "bool-setting", name = "startup-test-bool-setting", setting_type = "startup", default_value = true},
  {type = "int-setting", name = "startup-test-int-setting", setting_type = "startup", default_value = 123},
  {type = "int-setting", name = "startup-large-test-int-setting", setting_type = "startup", default_value = 69420},
  {type = "double-setting", name = "startup-test-double-setting", setting_type = "startup", default_value = 1.1},
  {type = "string-setting", name = "startup-test-string-setting", setting_type = "startup", default_value = "foo"},
  {type = "color-setting", name = "startup-color-setting", setting_type = "startup", default_value = {r = 1, g = 0.5, b = 0}},

  {type = "bool-setting", name = "runtime-global-test-bool-setting", setting_type = "runtime-global", default_value = true},
  {type = "int-setting", name = "runtime-global-test-int-setting", setting_type = "runtime-global", default_value = 123},
  {type = "int-setting", name = "runtime-global-large-test-int-setting", setting_type = "runtime-global", default_value = 69420},
  {type = "double-setting", name = "runtime-global-test-double-setting", setting_type = "runtime-global", default_value = 1.1},
  {type = "string-setting", name = "runtime-global-test-string-setting", setting_type = "runtime-global", default_value = "foo"},
  {type = "color-setting", name = "runtime-global-color-setting", setting_type = "runtime-global", default_value = {r = 1, g = 0.5, b = 0}},

  {type = "bool-setting", name = "runtime-per-user-test-bool-setting", setting_type = "runtime-per-user", default_value = true},
  {type = "int-setting", name = "runtime-per-user-test-int-setting", setting_type = "runtime-per-user", default_value = 123},
  {type = "int-setting", name = "runtime-per-user-large-test-int-setting", setting_type = "runtime-per-user", default_value = 69420},
  {type = "double-setting", name = "runtime-per-user-test-double-setting", setting_type = "runtime-per-user", default_value = 1.1},
  {type = "string-setting", name = "runtime-per-user-test-string-setting", setting_type = "runtime-per-user", default_value = "foo"},
  {type = "color-setting", name = "runtime-per-user-color-setting", setting_type = "runtime-per-user", default_value = {r = 1, g = 0.5, b = 0}},
})
