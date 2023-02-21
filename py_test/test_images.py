import docker_pyo3
from docker_pyo3.image import Images,Image
import pytest
import os
# Images Endpoints


here = os.path.abspath(os.path.dirname(__file__))

def test_images_init(docker):
    """images collection accessor"""
    x = docker.images()
    assert isinstance(x,Images)
    pass

def test_images_pull(docker):
    """pull an image from a remote"""
    x = docker.images().pull(image='busybox')
    assert isinstance(x,list)

def test_images_pull_bad(docker):
    """pulling a bad image fails with SystemError"""
    with pytest.raises(SystemError):
        docker.images().pull(image="asldfkjasd;lfk")

def test_images_list(docker, image_pull):
    """we can list images"""
    local_images = docker.images().list()
    assert isinstance(local_images, list)
    assert len(local_images) > 0
    pass


def test_images_prune(image_pull,docker):
    """prune dangling images"""
    start_images = docker.images().list()
    docker.images().prune()
    pass

def test_images_build(docker):
    """ we can build an image"""

    path =  os.path.join(here,'Dockerfile')

    with open(path,'w') as f:
        print("FROM busybox",file=f)
        print("COPY conftest.py /", file=f)

    try:
        x = docker.images().build(path=here,dockerfile='Dockerfile',tag='test-image')
    except Exception as e:
        raise(e)
    finally:
        os.unlink(path)
        docker.images().get('test-image').delete()

def test_images_get(image_pull, docker):
    """we can get and inspect images by Id and name"""
    x = docker.images().list();
    image = docker.images().get(x[0].get('Id'))
    assert isinstance(image, Image)
    image.inspect()
    
    image = docker.images().get('busybox')
    assert isinstance(image, Image)
    
    image.inspect()
    pass

def test_images_get_bad(docker):
    """non existent image interface fails"""
    
    image = docker.images().get("DSDFLKJ")
    with pytest.raises(SystemError):
        image.inspect()
        


def test_image_name(docker, image_pull):
    """images have a name"""
    try:
        image = docker.images().get('busybox')
        assert image.name() == 'busybox'
    except Exception as e:
        raise e



# def test_image_delete(docker):
#     """we can delete images"""    
#     try:
#         docker.images().pull('busybox')
#         image = docker.images().get('busybox')
#         image.delete()
#         with pytest.raises(SystemError):
#             image.inspect()
#     except Exception as e:
#         raise e
        

def test_image_export(docker, image_pull):
    """we can export images"""
    
    
    try:
        image = docker.images().get('busybox')
        image.export(path="busybox.tar")
        assert os.path.exists("busybox.tar")
    except Exception as e:
        raise e
    finally: 
        os.unlink("busybox.tar")

    
def test_image_tag(docker, image_pull):
    """we can tag images"""
    
    try:
        image = docker.images().get('busybox')    
        image.tag("test_tag")
    except Exception as e:
        raise e
    finally:
        docker.images().get("test_tag").delete()

    

    






