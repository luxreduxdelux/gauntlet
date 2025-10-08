bl_info = {
    "name"    : "Gauntlet Complex Editor",
    "blender" : (2, 80, 0)
}

#================================================================

import os
import json
import bpy
from bpy_extras.io_utils import ExportHelper, ImportHelper
from bpy.props import StringProperty, BoolProperty, EnumProperty
from bpy.types import Operator

#================================================================

def internal_name_property(class_name, name):
    return "gauntlet_" + class_name + "_" + name

def property_object(name, info):
    return bpy.props.PointerProperty(
        name = name,
        description = info,
        type = bpy.types.Object
    )

def property_boolean(name, info, default):
    return bpy.props.BoolProperty(
        name = name,
        description = info,
        default = default
    )

def property_integer(name, info, default):
    return bpy.props.IntProperty(
        name = name,
        description = info,
        default = default
    )

def property_decimal(name, info, default):
    return bpy.props.FloatProperty(
        name = name,
        description = info,
        default = default
    )

def property_string(name, info, default):
    return bpy.props.StringProperty(
        name = name,
        description = info,
        default = default
    )
    
def property_color(name, info, default):
    return bpy.props.FloatVectorProperty(
        name = name,
        description = info,
        default = default,
        subtype = "COLOR",
        soft_min = 0.0,
        soft_max = 1.0
    )

def property_enumerator(name, info, list, default):
    return bpy.props.EnumProperty(
        name = name,
        description = info,
        items = list,
        default = default
    )

ENTITY_LIST = {}

def load_entity_info(path):
    entity_list = {}

    for file_path in os.listdir(path):
        entity_name = file_path.split(".")[0]
        entity_data = {}

        with open(path + "/" + file_path) as file:
            entity_data = json.load(file)
        
            for field in entity_data["data"]:
                entity_field = entity_data["data"][field]
                entity_type  = entity_field["type"]

                match entity_type:
                    case "Object":
                        entity_data["data"][field] = property_object(entity_field["name"], entity_field["info"])
                    case "Boolean":
                        entity_data["data"][field] = property_boolean(entity_field["name"], entity_field["info"], entity_field["data"])
                    case "Integer":
                        entity_data["data"][field] = property_integer(entity_field["name"], entity_field["info"], entity_field["data"])
                    case "Decimal":
                        entity_data["data"][field] = property_decimal(entity_field["name"], entity_field["info"], entity_field["data"])
                    case "String":
                        entity_data["data"][field] = property_string(entity_field["name"], entity_field["info"], entity_field["data"])
                    case "Color":
                        entity_data["data"][field] = property_color(entity_field["name"], entity_field["info"], (
                            entity_field["data"][0] // 255.0,
                            entity_field["data"][1] // 255.0,
                            entity_field["data"][2] // 255.0
                        ))
                    case "Enumerator":
                        pick = []

                        for choice in entity_field["pick"]:
                            pick.append((
                                choice[0],
                                choice[0],
                                choice[1],
                            ))

                        entity_data["data"][field] = property_enumerator(entity_field["name"], entity_field["info"], pick, entity_field["data"])

        entity_list[entity_name] = entity_data

    return entity_list

def create_entity_list(path):
    scene = bpy.context.scene
    
    # remove later, only for debugging.
    scene.panel_entity_list.clear()
    
    global ENTITY_LIST
    
    ENTITY_LIST = load_entity_info(path)
    
    # for each entity in the entity list...
    for entity_name in ENTITY_LIST:
        # get the entity's data, add them to the UI entity list.
        entity_data = ENTITY_LIST[entity_name]
        item      = scene.panel_entity_list.add()
        item.name = entity_name
        
        # for each entity property...
        for key in entity_data["data"]:
            # assign the entity property to the global Object type.
            value = entity_data["data"][key]
            setattr(bpy.types.Object, internal_name_property(entity_name, key), value)

def delete_entity_list():
    # for each entity in the entity list...
    for entity_name in ENTITY_LIST:
        # get the entity's data.
        entity_data = ENTITY_LIST[entity_name]
        
        # for each entity property...
        for key in entity_data["data"]:
            delattr(bpy.types.Object, internal_name_property(entity_name, key))

#================================================================

class PanelMain(bpy.types.Panel):
    bl_label       = "Gauntlet Complex"
    bl_idname      = "gauntlet.panel_main"
    bl_space_type  = "VIEW_3D"
    bl_region_type = "UI"
    bl_category    = "Gauntlet Complex"

    def draw(self, context):
        layout = self.layout
        scene  = context.scene

        box = layout.box()
        row = box.row()
        row.template_list(
            "PanelList",
            "",
            scene, "panel_entity_list",
            scene, "panel_entity_list_index",
        )

        box.operator("gauntlet.create_entity")
        
        object = context.active_object
        
        if len(ENTITY_LIST) > 0:
            if object != None:
                if "entity_class" in object:
                    entity_class_name = object["entity_class"]
                    entity_class_data = ENTITY_LIST[entity_class_name]
                    
                    if len(entity_class_data["data"]) > 0:
                        box = layout.box()
                    
                        for key in entity_class_data["data"]:
                            row = box.row()
                            row.prop(object, internal_name_property(entity_class_name, key))
        
        box = layout.box()
        row = box.row()
        row.operator("gauntlet.save_level")
        row = box.row()
        row.operator("gauntlet.load_info")

#================================================================

class OperatorSave(Operator, ExportHelper):
    """Save a level."""
    bl_idname = "gauntlet.save_level"
    bl_label  = "Save Level"

    filename_ext = ".json"

    filter_glob: StringProperty(
        default="*.json",
        options={"HIDDEN"},
        maxlen=255,
    )

    def execute(self, context):
        # iterate through every entity and store in .JSON file.
        # also save current scene to a .GLB file.
        
        # export scene.
        model_path  = self.filepath.split("/")[0:-1]
        model_path  = "/".join(model_path)
        model_name  = self.filepath.split("/")[-1][:-5]
        model_list  = export_scene(model_path + "/" + model_name)
        
        with open(self.filepath, "w") as file:
            
            # TO-DO: export material data of level.
            
            # level data.
            entity_list = []
            entity_file = {
                "level": model_list,
                "entity_list": entity_list
            }
            
            # for each object in the scene...
            for object in bpy.data.objects:
                # if the object has an entity class...
                if "entity_class" in object:
                    point = {
                        "x":  object.location[0],
                        # TO-DO should this 0.01 epsilon be here at all?
                        "y":  object.location[2] + 0.01,
                        "z": -object.location[1],
                    }
                    angle = {
                        "x":  object.rotation_euler[2] * (180.0 / 3.14) + 90.0,
                        "y":  object.rotation_euler[1] * (180.0 / 3.14),
                        "z":  object.rotation_euler[0] * (180.0 / 3.14),
                    }
                    scale = {
                        "x": object.scale[0],
                        "y": object.scale[2],
                        "z": object.scale[1],
                    }
                    
                    # get the entity's class name.
                    entity_class_name = object["entity_class"]
                    entity_data  = {
                        "type"  : entity_class_name,
                        "name"  : object.name,
                        "point" : point,
                        "angle" : angle,
                        "scale" : scale
                    }
                    
                    # get the entity's class data.
                    entity_class_data = ENTITY_LIST[entity_class_name]
                    
                    # for each class data field...
                    for key in entity_class_data["data"]:
                        # get the entity's value for that data key.
                        data = getattr(object, internal_name_property(entity_class_name, key))
                        
                        import math
                        
                        # if the data value is an object, use the object's name.
                        if type(data) is bpy.types.Object:
                            entity_data[key] = data.name
                        elif "Color" in str(type(data)):
                            entity_data[key] = {
                                "r": math.floor(data[0] * 255.0),
                                "g": math.floor(data[1] * 255.0),
                                "b": math.floor(data[2] * 255.0),
                                "a": 255
                            }
                        else:
                            entity_data[key] = data
                    
                    # add entity to entity list.        
                    entity_list.append(entity_data)
            
            # write level file.
            json.dump(entity_file, file, indent = 2)

        return {"FINISHED"}

#================================================================

class OperatorInfo(Operator, ImportHelper):
    """Load the entity definition info folder."""
    bl_idname = "gauntlet.load_info"
    bl_label  = "Load Info"

    filepath: bpy.props.StringProperty(
        name = "Folder Path",
        subtype = "DIR_PATH"
    )

    def execute(self, context):
        path = self.filepath.split("/")[0:-1]
        path = "/".join(path)
        
        delete_entity_list()
        create_entity_list(path)

        return {"FINISHED"}

#================================================================

def export_scene(file_path):
    bpy.ops.object.select_all(action='DESELECT')
    
    path  = os.path.dirname(file_path)
    name  = file_path.split("/")[-1]
    index = 0
    model = []

    for object in bpy.data.objects:
        if object.type == "MESH":
            path_current = path + "/" + name + "_" + str(index)
            path_mesh = path_current + ".glb"
            path_data = path_current + ".meta"
            
            model.append(name + "_" + str(index) + ".glb")
            
            with open(path_data, "w") as file:
                object_file = {
                    "texture": []
                }
                
                for slot in object.material_slots:
                    material = slot.material
                    
                    if material:
                        for node in material.node_tree.nodes:
                            if node.type == 'TEX_IMAGE':
                                if node.image:
                                    image_path = node.image.filepath
                                    image_path = image_path.split("video")
                                    image_path = "video" + image_path[1]
                                    object_file["texture"].append(image_path)
                
                json.dump(object_file, file, indent = 2)

            object.select_set(True)

            bpy.ops.export_scene.gltf(filepath=path_mesh, use_selection=True, export_image_format="NONE")

            object.select_set(False)
            
            index += 1
    
    return model

class OperatorSaveModel(Operator, ExportHelper):
    """Save a model."""
    bl_idname = "gauntlet.save_model"
    bl_label  = "Save Model"

    filename_ext = ""

    filter_glob: StringProperty(
        default="*.glb",
        options={"HIDDEN"},
        maxlen=255,
    )

    def execute(self, context):
        export_scene(self.filepath)

        return {"FINISHED"}

#================================================================

class OperatorCreateEntity(bpy.types.Operator):
    """Create the active entity in the list."""
    bl_idname = "gauntlet.create_entity"
    bl_label  = "Create Entity"

    def execute(self, context):
        scene = context.scene
        index = scene.panel_entity_list_index
        
        import mathutils

        # create entity.
        if index >= 0 and index < len(scene.panel_entity_list):
            which = scene.panel_entity_list[index]
            entity_class = ENTITY_LIST[which.name]
            bpy.ops.object.empty_add(type = "CUBE")
            object = context.active_object
            object.name = which.name
            object.show_name = True
            object.show_axis = True
            
            if entity_class["info"]:
                object.scale[0] = entity_class["info"]["scale"][2]
                object.scale[1] = entity_class["info"]["scale"][0]
                object.scale[2] = entity_class["info"]["scale"][1]
            else:
                object.scale[0] = 0.25
                object.scale[1] = 0.25
                object.scale[2] = 0.25
            
            object["entity_class"] = which.name
        
        return {'FINISHED'}

#================================================================

class PanelList(bpy.types.UIList):
    def draw_item(self, context, layout, data, item, icon, active_data, active_propname, index):
        if self.layout_type in {'DEFAULT', 'COMPACT'}:
            layout.label(text=item.name)
        elif self.layout_type in {'GRID'}:
            layout.alignment = 'CENTER'
            layout.label(text="")

class PanelListItem(bpy.types.PropertyGroup):
    name: bpy.props.StringProperty(name="Name", default="Untitled")

#================================================================

# Register everything
classes = (
    PanelList,
    PanelListItem,
    PanelMain,
    OperatorCreateEntity,
    OperatorSave,
    OperatorSaveModel,
    OperatorInfo,
)

def register():
    for cls in classes:
        bpy.utils.register_class(cls)

    bpy.types.Scene.panel_entity_list       = bpy.props.CollectionProperty(type=PanelListItem)
    bpy.types.Scene.panel_entity_list_index = bpy.props.IntProperty(default=-1)

def unregister():    
    for cls in reversed(classes):
        bpy.utils.unregister_class(cls)

    del bpy.types.Scene.panel_entity_list
    del bpy.types.Scene.panel_entity_list_index

    delete_entity_list()

if __name__ == "__main__":
    register()