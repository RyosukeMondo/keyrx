/// Re-exports core BenchmarkService for UI use.
///
/// All benchmark operations are handled by core_logic.
/// Use core_logic BenchmarkService directly.
library;

export 'package:core_logic/services/benchmark_service.dart'
    show
        BenchmarkData,
        BenchmarkService,
        BenchmarkServiceImpl,
        BenchmarkServiceResult;
