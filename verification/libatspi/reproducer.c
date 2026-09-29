#define _GNU_SOURCE

#include <atspi/atspi.h>
#include <dbus/dbus.h>
#include <dlfcn.h>
#include <glib-object.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned finalized;
static unsigned calls;

static void
state_finalized (gpointer data, GObject *where_the_object_was)
{
  (void) data;
  (void) where_the_object_was;
  finalized++;
}

dbus_bool_t
_atspi_dbus_call (gpointer object,
                  const char *interface,
                  const char *method,
                  GError **error,
                  const char *type,
                  ...)
{
  AtspiAccessible *accessible = ATSPI_ACCESSIBLE (object);
  dbus_uint32_t words[2] = { 0, 0 };
  GArray **result;
  va_list arguments;

  (void) interface;
  (void) error;
  if (strcmp (method, "GetState") != 0 || strcmp (type, "=>au") != 0)
    abort ();

  va_start (arguments, type);
  result = va_arg (arguments, GArray **);
  va_end (arguments);

  words[ATSPI_STATE_FOCUSED / 32] |=
      ((dbus_uint32_t) 1) << (ATSPI_STATE_FOCUSED % 32);
  *result = g_array_sized_new (FALSE, TRUE, sizeof (dbus_uint32_t), 2);
  g_array_append_vals (*result, words, 2);

  /* Reproduce the owner release observed during a reentrant GetState call. */
  g_clear_object (&accessible->states);
  calls++;
  return TRUE;
}

static void
require_isolated_library (void)
{
  const char *expected = getenv ("RTUI_EXPECTED_LIBATSPI_DIR");
  Dl_info info = { 0 };

  if (!expected || !dladdr ((void *) atspi_state_set_contains, &info) ||
      !info.dli_fname || !g_str_has_prefix (info.dli_fname, expected))
    {
      fprintf (stderr, "libatspi did not resolve from the isolated build: %s\n",
               info.dli_fname ? info.dli_fname : "unknown");
      exit (2);
    }
  printf ("LIBATSPI %s\n", info.dli_fname);
}

int
main (int argc, char **argv)
{
  AtspiAccessible *accessible;
  AtspiStateSet *set;
  gboolean found = FALSE;

  if (argc != 2 ||
      (strcmp (argv[1], "contains") != 0 && strcmp (argv[1], "get-states") != 0))
    return 64;

  require_isolated_library ();
  accessible = g_object_new (ATSPI_TYPE_ACCESSIBLE, NULL);
  set = _atspi_state_set_new_internal (accessible, 0);
  accessible->cached_properties = 0;
  accessible->states = set;
  g_object_weak_ref (G_OBJECT (set), state_finalized, NULL);

  if (strcmp (argv[1], "contains") == 0)
    {
      found = atspi_state_set_contains (set, ATSPI_STATE_FOCUSED);
    }
  else
    {
      GArray *states = atspi_state_set_get_states (set);
      guint index;

      for (index = 0; states && index < states->len; index++)
        if (g_array_index (states, AtspiStateType, index) == ATSPI_STATE_FOCUSED)
          found = TRUE;
      if (states)
        g_array_free (states, TRUE);
    }

  if (!found || calls != 1 || finalized != 1 || accessible->states != NULL)
    {
      fprintf (stderr,
               "unexpected result: found=%d calls=%u finalized=%u owner=%p\n",
               found, calls, finalized, (void *) accessible->states);
      return 3;
    }

  printf ("PASS %s retained the state set through reentrant owner release\n",
          argv[1]);
  g_object_unref (accessible);
  return 0;
}
