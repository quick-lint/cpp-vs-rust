#include <benchmark/benchmark.h>
#include <cstddef>
#include <cstdlib>
#include <dlfcn.h>
#include <string_view>

using namespace std::literals::string_view_literals;

namespace {
struct c_api {
  enum qljs_severity {
    qljs_severity_error = 1,
    qljs_severity_warning = 2,
  };

  struct diagnostic {
    const char* message;
    char code[6];  // null-terminated
    qljs_severity severity;
    // Offsets count UTF-16 code units.
    int begin_offset;
    int end_offset;
  };

  struct document;

  static c_api load() {
    const char* dll_path = std::getenv("CPP_VS_RUST_DLL");
    if (!dll_path || dll_path[0] == '\0') {
      std::fprintf(stderr, "fatal: CPP_VS_RUST_DLL environment variable must be set\n");
      std::exit(1);
    }
    return c_api::load(dll_path);
  }

  static c_api load(const char *dll_path) {
    void *dll = ::dlopen(dll_path, /*flags=*/RTLD_LAZY);
    if (!dll) {
      std::fprintf(stderr, "fatal: could not load %s: %s\n", dll_path, ::dlerror());
      std::exit(1);
    }

    c_api api;
#define LOAD_SYM(sym) do { \
  const char *full_sym = "qljs_web_demo_" #sym; \
  api.sym = (decltype(api.sym)) ::dlsym(dll, full_sym); \
  if (!api.sym) { \
      std::fprintf(stderr, "fatal: could not load symbol %s from %s\n", full_sym, dll_path); \
      std::exit(1); \
  } \
} while (false)
    LOAD_SYM(create_document);
    LOAD_SYM(destroy_document);
    LOAD_SYM(set_text);
    LOAD_SYM(lint);
#undef LOAD_SYM
    return api;
  }

  document* (*create_document)(void);
  void (*destroy_document)(document* document);
  void (*set_text)(document* document, const void* text_utf_8, std::size_t text_byte_count);
  const diagnostic* (*lint)(document* document);
};

void benchmark_lex(::benchmark::State &state, std::string_view raw_source) {
  c_api api = c_api::load();

  c_api::document* doc = api.create_document();
  api.set_text(doc, raw_source.data(), raw_source.size());

  for (auto _ : state) {
    ::benchmark::DoNotOptimize(api.lint(doc));
  }

  double bytes_per_iteration = static_cast<double>(raw_source.size() + 1);
  double iteration_count = static_cast<double>(state.iterations());
  state.counters["bytes"] = ::benchmark::Counter(
      bytes_per_iteration * iteration_count, ::benchmark::Counter::kIsRate);
  state.counters["byte"] = ::benchmark::Counter(
      bytes_per_iteration * iteration_count,
      ::benchmark::Counter::kIsRate | ::benchmark::Counter::kInvert);

  api.destroy_document(doc);
}

BENCHMARK_CAPTURE(benchmark_lex, jquery_snippet,
                  R"(/*!
 * Copyright JS Foundation and other contributors
 * Released under the MIT license
 * https://jquery.org/license
 *
 * Date: 2020-05-04T22:49Z
 */
function buildFragment( elems, context, scripts, selection, ignored ) {
	var elem, tmp, tag, wrap, attached, j,
		fragment = context.createDocumentFragment(),
		nodes = [],
		i = 0,
		l = elems.length;

	for ( ; i < l; i++ ) {
		elem = elems[ i ];

		if ( elem || elem === 0 ) {

			// Add nodes directly
			if ( toType( elem ) === "object" ) {
)"sv);
}
