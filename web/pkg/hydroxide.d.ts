/* tslint:disable */
/* eslint-disable */

export class WasmRenderer {
    free(): void;
    [Symbol.dispose](): void;
    add_cube(x: number, y: number, z: number, size: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): number;
    add_mesh_stl(stl_bytes: Uint8Array, x: number, y: number, z: number, size: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): number;
    add_sphere(x: number, y: number, z: number, radius: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): number;
    export_scene(width: number, height: number, fov: number, cam_x: number, cam_y: number, cam_z: number, target_x: number, target_y: number, target_z: number, focus_distance: number, aperture: number, samples: number, termination_prob: number): Uint8Array;
    get_object_info(index: number): Float64Array;
    constructor();
    object_count(): number;
    outline(object_index: number, width: number, height: number, fov: number, cam_x: number, cam_y: number, cam_z: number, target_x: number, target_y: number, target_z: number, focus_distance: number, _aperture: number, radius: number): Uint8Array;
    pick(pixel_x: number, pixel_y: number, width: number, height: number, fov: number, cam_x: number, cam_y: number, cam_z: number, target_x: number, target_y: number, target_z: number, focus_distance: number, _aperture: number): number;
    pick_distance(pixel_x: number, pixel_y: number, width: number, height: number, fov: number, cam_x: number, cam_y: number, cam_z: number, target_x: number, target_y: number, target_z: number): number;
    remove_object(index: number): void;
    render(width: number, height: number, fov: number, cam_x: number, cam_y: number, cam_z: number, target_x: number, target_y: number, target_z: number, samples: number, termination_prob: number, focus_distance: number, aperture: number): Uint8Array;
    restore(bytes: Uint8Array): void;
    set_sky(index: number): void;
    set_sky_hdr(hdr_index: number, bytes: Uint8Array): void;
    set_sky_hdr_bytes(bytes: Uint8Array): void;
    sky_count(): number;
    sky_name(index: number): string;
    snapshot(): Uint8Array;
    update_cube(index: number, x: number, y: number, z: number, size: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): void;
    update_mesh(index: number, new_cx: number, new_cy: number, new_cz: number, new_size: number, rot_x: number, rot_y: number, rot_z: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): void;
    update_mesh_material(index: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): void;
    update_sphere(index: number, x: number, y: number, z: number, radius: number, mat_type: number, r: number, g: number, b: number, fuzz: number, refractive_index: number): void;
}

export function initThreadPool(num_threads: number): Promise<any>;

export class wbg_rayon_PoolBuilder {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    build(): void;
    numThreads(): number;
    receiver(): number;
}

export function wbg_rayon_start_worker(receiver: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly __wbg_wasmrenderer_free: (a: number, b: number) => void;
    readonly wasmrenderer_add_cube: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => number;
    readonly wasmrenderer_add_mesh_stl: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => number;
    readonly wasmrenderer_add_sphere: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => number;
    readonly wasmrenderer_export_scene: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => void;
    readonly wasmrenderer_get_object_info: (a: number, b: number, c: number) => void;
    readonly wasmrenderer_new: () => number;
    readonly wasmrenderer_object_count: (a: number) => number;
    readonly wasmrenderer_outline: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => void;
    readonly wasmrenderer_pick: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => number;
    readonly wasmrenderer_pick_distance: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => number;
    readonly wasmrenderer_remove_object: (a: number, b: number) => void;
    readonly wasmrenderer_render: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => void;
    readonly wasmrenderer_restore: (a: number, b: number, c: number) => void;
    readonly wasmrenderer_set_sky: (a: number, b: number) => void;
    readonly wasmrenderer_set_sky_hdr: (a: number, b: number, c: number, d: number) => void;
    readonly wasmrenderer_set_sky_hdr_bytes: (a: number, b: number, c: number) => void;
    readonly wasmrenderer_sky_count: (a: number) => number;
    readonly wasmrenderer_sky_name: (a: number, b: number, c: number) => void;
    readonly wasmrenderer_snapshot: (a: number, b: number) => void;
    readonly wasmrenderer_update_cube: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
    readonly wasmrenderer_update_mesh: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => void;
    readonly wasmrenderer_update_mesh_material: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly wasmrenderer_update_sphere: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
    readonly __wbg_wbg_rayon_poolbuilder_free: (a: number, b: number) => void;
    readonly initThreadPool: (a: number) => number;
    readonly wbg_rayon_poolbuilder_build: (a: number) => void;
    readonly wbg_rayon_poolbuilder_numThreads: (a: number) => number;
    readonly wbg_rayon_poolbuilder_receiver: (a: number) => number;
    readonly wbg_rayon_start_worker: (a: number) => void;
    readonly memory: WebAssembly.Memory;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number) => void;
    readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
    readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
