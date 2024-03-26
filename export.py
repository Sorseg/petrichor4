import bpy

objects = bpy.context.scene.objects


def is_server(obj: bpy.types.Object) -> bool:
    """This logic needs to be synchronized with `petri_obj.rs`"""
    return obj.name.lower().startswith(("collider", "info_"))

def is_client(obj: bpy.types.Object) -> bool:
    """This logic needs to be synchronized with `petri_obj.rs`"""
    # todo: filter out server only colliders, triggers, etc
    return True

def export_server(fname: str):
    for obj in objects:
        obj.select_set(is_server(obj))
        
    bpy.ops.wm.obj_export(
        filepath=fname,
        check_existing=False,
        export_selected_objects=True,
        export_uv=False,
        export_normals=False,
        export_materials=False,
        export_triangulated_mesh=True,
        
    )
    
def export_client(fname: str):
    for obj in objects:
        obj.select_set(is_client(obj))

    bpy.ops.export_scene.gltf(
        filepath=fname,
        check_existing=False,
        use_selection=True,
        export_lights=True,
        export_extras=True
    )

if not bpy.data.filepath:
    raise RuntimeError("Save the file first")

export_server(f"{bpy.data.filepath}.obj")
export_client(f"{bpy.data.filepath}.glb")
